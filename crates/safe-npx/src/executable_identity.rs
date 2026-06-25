//! M2 executable byte identity for selected bins and generated shims.
//!
//! Identity is filesystem and byte inspection only. It never invokes package
//! binaries, generated shims, shells, package managers, lifecycle scripts, or
//! node.

use crate::{
    ClosureDecision, ExecutableFileIdentity, ExecutableFileSource, ExtractedRootArtifact, M2Reason,
    SelectedPackageBin,
};
use sha2::{Digest, Sha512};
use std::fs;
use std::path::Path;

/// Digest algorithm used for executable file identity.
pub const EXECUTABLE_DIGEST_ALGORITHM: &str = "sha512";

/// Error returned when executable identity cannot be proven.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableIdentityError {
    /// Conservative M2 decision for this identity failure.
    pub decision: ClosureDecision,
    /// Stable M2 reason for this identity failure.
    pub reason: M2Reason,
    /// Deterministic detail for tests and diagnostics.
    pub detail: String,
}

impl ExecutableIdentityError {
    /// Build an unsupported-closure identity failure.
    fn unsupported(detail: impl Into<String>) -> Self {
        Self {
            decision: M2Reason::UnsupportedClosure.refusal_decision(),
            reason: M2Reason::UnsupportedClosure,
            detail: detail.into(),
        }
    }
}

/// Build root-artifact executable identity for a selected package bin.
pub fn identify_selected_bin(
    artifact: &ExtractedRootArtifact,
    selected_bin: &SelectedPackageBin,
) -> Result<ExecutableFileIdentity, ExecutableIdentityError> {
    let root = canonicalize_path(&artifact.extraction_root, "extraction root")?;
    let candidate = artifact.extraction_root.join(&selected_bin.relative_path);
    let candidate = canonicalize_path(&candidate, "selected bin")?;

    if !candidate.starts_with(&root) {
        return Err(ExecutableIdentityError::unsupported(
            "selected bin path escapes verified extraction root",
        ));
    }

    let metadata = fs::metadata(&candidate).map_err(|error| {
        ExecutableIdentityError::unsupported(format!("could not inspect selected bin: {error}"))
    })?;
    if !metadata.is_file() {
        return Err(ExecutableIdentityError::unsupported(
            "selected bin path is not a regular file",
        ));
    }

    let bytes = fs::read(&candidate).map_err(|error| {
        ExecutableIdentityError::unsupported(format!("could not read selected bin bytes: {error}"))
    })?;

    Ok(ExecutableFileIdentity {
        relative_path: selected_bin.relative_path.clone(),
        digest_algorithm: EXECUTABLE_DIGEST_ALGORITHM.to_string(),
        digest: sha512_hex(&bytes),
        source: ExecutableFileSource::RootArtifact,
    })
}

/// Refuse generated shim candidates until deterministic shim bytes are modeled.
pub fn refuse_generated_shim_candidate(
    relative_path: &str,
) -> Result<ExecutableFileIdentity, ExecutableIdentityError> {
    Err(ExecutableIdentityError::unsupported(format!(
        "generated shim identity is not modeled for M2: {relative_path}"
    )))
}

/// Canonicalize a path into an existing filesystem location.
fn canonicalize_path(
    path: &Path,
    label: &str,
) -> Result<std::path::PathBuf, ExecutableIdentityError> {
    path.canonicalize().map_err(|error| {
        ExecutableIdentityError::unsupported(format!("{label} is unavailable: {error}"))
    })
}

/// Compute a lowercase SHA-512 hex digest.
fn sha512_hex(bytes: &[u8]) -> String {
    Sha512::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(test)]
/// Tests for executable byte identity.
mod tests {
    use super::*;
    use crate::{
        ArtifactIdentity, BinSelectionKind, ExtractedPackageMetadata, ExtractedRootArtifact,
        SelectedPackageBin,
    };
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    /// Verifies direct selected-bin bytes are tied to the verified root artifact.
    fn direct_bin_identity_records_digest() {
        let workspace = TempRoot::new();
        workspace.write_file("bin/create.js", b"console.log('direct');\n");
        let selected = selected_bin("create-example", "bin/create.js");

        let identity = identify_selected_bin(&artifact_at(workspace.path()), &selected)
            .expect("direct bin should identify");

        assert_eq!(identity.relative_path, "bin/create.js");
        assert_eq!(identity.digest_algorithm, EXECUTABLE_DIGEST_ALGORITHM);
        assert_eq!(identity.digest, sha512_hex(b"console.log('direct');\n"));
        assert_eq!(identity.source, ExecutableFileSource::RootArtifact);
    }

    #[test]
    /// Verifies package bin-object selections produce root-artifact byte identity.
    fn package_bin_object_identity_records_digest() {
        let workspace = TempRoot::new();
        workspace.write_file("bin/object.js", b"object-bin");
        let mut metadata = metadata_with_bins([("create-example", "bin/object.js")]);
        metadata.name = Some("create-example".to_string());
        let selected = crate::select_package_bin(&metadata).expect("object bin should select");

        let identity = identify_selected_bin(&artifact_at(workspace.path()), &selected)
            .expect("object bin should identify");

        assert_eq!(identity.relative_path, "bin/object.js");
        assert_eq!(identity.digest, sha512_hex(b"object-bin"));
    }

    #[test]
    /// Verifies package string-bin selections use normalized metadata identity.
    fn package_bin_string_identity_records_digest() {
        let workspace = TempRoot::new();
        workspace.write_file("cli.js", b"string-bin");
        let mut metadata = metadata_with_bins([("create-example", "cli.js")]);
        metadata.name = Some("@scope/create-example".to_string());
        let selected = crate::select_package_bin(&metadata).expect("string bin should select");

        let identity = identify_selected_bin(&artifact_at(workspace.path()), &selected)
            .expect("string bin should identify");

        assert_eq!(selected.name, "create-example");
        assert_eq!(identity.relative_path, "cli.js");
        assert_eq!(identity.digest, sha512_hex(b"string-bin"));
    }

    #[test]
    /// Verifies generated shims fail closed until deterministic bytes are modeled.
    fn generated_shim_candidate_is_refused() {
        let error = refuse_generated_shim_candidate(".safe-npx/shims/create-example")
            .expect_err("generated shim should be refused");

        assert_eq!(error.decision, ClosureDecision::ExecutionRefused);
        assert_eq!(error.reason, M2Reason::UnsupportedClosure);
        assert!(error
            .detail
            .contains("generated shim identity is not modeled"));
    }

    #[test]
    /// Verifies selected bin paths cannot escape the verified extraction root.
    fn bin_path_escape_is_refused() {
        let workspace = TempRoot::new();
        let outside_file = format!("safe-npx-outside-{}.js", next_temp_id());
        let outside_path = workspace
            .path()
            .parent()
            .expect("temp root should have parent")
            .join(&outside_file);
        fs::write(&outside_path, b"outside").expect("outside fixture should be writable");
        let selected = selected_bin("create-example", &format!("../{outside_file}"));

        let error = identify_selected_bin(&artifact_at(workspace.path()), &selected)
            .expect_err("escape should be refused");

        assert_eq!(error.decision, ClosureDecision::ExecutionRefused);
        assert_eq!(error.reason, M2Reason::UnsupportedClosure);
        assert!(error.detail.contains("escapes verified extraction root"));
        fs::remove_file(outside_path).expect("outside fixture should be removable");
    }

    #[cfg(unix)]
    #[test]
    /// Verifies symlinked selected bins cannot point outside the verified root.
    fn selected_bin_symlink_escape_is_refused() {
        let workspace = TempRoot::new();
        let outside = TempRoot::new();
        outside.write_file("outside.js", b"outside");
        std::os::unix::fs::symlink(
            outside.path().join("outside.js"),
            workspace.path().join("bin.js"),
        )
        .expect("test symlink should be creatable");
        let selected = selected_bin("create-example", "bin.js");

        let error = identify_selected_bin(&artifact_at(workspace.path()), &selected)
            .expect_err("symlink escape should be refused");

        assert_eq!(error.reason, M2Reason::UnsupportedClosure);
        assert!(error.detail.contains("escapes verified extraction root"));
    }

    fn selected_bin(name: &str, relative_path: &str) -> SelectedPackageBin {
        SelectedPackageBin {
            name: name.to_string(),
            relative_path: relative_path.to_string(),
            selection: BinSelectionKind::SingleDeclaredBin,
        }
    }

    fn artifact_at(extraction_root: &Path) -> ExtractedRootArtifact {
        ExtractedRootArtifact {
            artifact: ArtifactIdentity {
                name: "create-example".to_string(),
                version: "1.2.3".to_string(),
                integrity: "sha512-fixture".to_string(),
                digest_algorithm: "sha512".to_string(),
                digest: "fixture".to_string(),
            },
            extraction_root: extraction_root.to_path_buf(),
            metadata: metadata_with_bins([("create-example", "bin/create.js")]),
        }
    }

    fn metadata_with_bins<const N: usize>(
        bins: [(&'static str, &'static str); N],
    ) -> ExtractedPackageMetadata {
        ExtractedPackageMetadata {
            name: Some("create-example".to_string()),
            version: Some("1.2.3".to_string()),
            bins: BTreeMap::from(bins.map(|(name, path)| (name.to_string(), path.to_string()))),
            lifecycle_scripts: BTreeMap::new(),
            dependency_declarations: Vec::new(),
            package_json_path: PathBuf::from("package/package.json"),
        }
    }

    struct TempRoot {
        path: PathBuf,
    }

    impl TempRoot {
        /// Create a unique temporary root.
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "safe-npx-executable-{}-{}-{}",
                std::process::id(),
                temp_millis(),
                next_temp_id(),
            ));
            fs::create_dir_all(&path).expect("temp root should be creatable");
            Self { path }
        }

        /// Return the temporary root path.
        fn path(&self) -> &Path {
            &self.path
        }

        /// Write a file below the temporary root.
        fn write_file(&self, relative_path: &str, contents: &[u8]) {
            let path = self.path.join(relative_path);
            fs::create_dir_all(path.parent().expect("test file should have parent"))
                .expect("test parent should be creatable");
            fs::write(path, contents).expect("test file should be writable");
        }
    }

    impl Drop for TempRoot {
        /// Remove the temporary root.
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn next_temp_id() -> u64 {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        NEXT_ID.fetch_add(1, Ordering::SeqCst)
    }

    fn temp_millis() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_millis()
    }
}
