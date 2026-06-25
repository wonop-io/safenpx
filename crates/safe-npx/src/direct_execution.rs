//! M2 direct-extract execution prototype for local fixture packages.
//!
//! This module is intentionally fixture-only. It never invokes npm, npx,
//! package-manager install commands, lifecycle scripts, or shell fallbacks.
//! It is compiled only for tests so package-controlled marker files cannot
//! enable production execution.

use crate::process_boundary::ProcessInvocation;
use crate::{
    assess_static_closure_blockers, identify_selected_bin, select_package_bin,
    ClosureCommandIdentity, ClosureDecision, CommandIntent, ExecutableFileIdentity,
    ExtractedRootArtifact, M2Reason, PackageSpecParse,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Marker file required before the direct execution prototype may run.
pub const LOCAL_FIXTURE_MARKER: &str = ".safe-npx-local-fixture";

/// Result of the fixture-only direct execution prototype.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DirectExecutionOutcome {
    /// Conservative closure decision.
    pub decision: ClosureDecision,
    /// Stable reasons supporting `decision`.
    pub reasons: Vec<M2Reason>,
    /// Command identity requested by the caller.
    pub command: ClosureCommandIdentity,
    /// Selected executable identity, when byte identity was proven.
    pub selected_bin: Option<ExecutableFileIdentity>,
    /// Canonical cwd used for the fixture process.
    pub cwd: Option<PathBuf>,
    /// Environment boundary used for the fixture process.
    pub environment: DirectExecutionEnvironment,
    /// Exit code when the fixture process ran.
    pub exit_code: Option<i32>,
    /// Captured stdout when the fixture process ran.
    pub stdout: Vec<u8>,
    /// Captured stderr when the fixture process ran.
    pub stderr: Vec<u8>,
    /// Deterministic diagnostic detail.
    pub detail: String,
}

/// Environment boundary for fixture execution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DirectExecutionEnvironment {
    /// Whether inherited caller environment variables were cleared.
    pub inherited_environment_cleared: bool,
    /// Explicit environment variables supplied to the fixture.
    pub variables: BTreeMap<String, String>,
}

impl DirectExecutionOutcome {
    /// Build a refusal outcome before package code can run.
    fn refused(
        intent: &CommandIntent,
        reason: M2Reason,
        selected_bin: Option<ExecutableFileIdentity>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            decision: ClosureDecision::ExecutionRefused,
            reasons: vec![reason],
            command: ClosureCommandIdentity::from(intent),
            selected_bin,
            cwd: None,
            environment: fixture_environment(),
            exit_code: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
            detail: detail.into(),
        }
    }
}

/// Execute a verified local fixture package from its extracted root.
pub fn execute_local_fixture_package(
    artifact: &ExtractedRootArtifact,
    intent: &CommandIntent,
) -> DirectExecutionOutcome {
    if !artifact
        .extraction_root
        .join(LOCAL_FIXTURE_MARKER)
        .is_file()
    {
        return DirectExecutionOutcome::refused(
            intent,
            M2Reason::UnsupportedClosure,
            None,
            "direct execution prototype only accepts marked local fixtures",
        );
    }

    let PackageSpecParse::Supported(spec) = &intent.package_spec else {
        return DirectExecutionOutcome::refused(
            intent,
            M2Reason::UnsupportedClosure,
            None,
            "direct execution prototype requires exact-version package intent",
        );
    };
    if artifact.artifact.name != spec.name || artifact.artifact.version != spec.version {
        return DirectExecutionOutcome::refused(
            intent,
            M2Reason::UnsupportedClosure,
            None,
            "artifact identity does not match requested exact package coordinates",
        );
    }

    let blocker_assessment = assess_static_closure_blockers(&artifact.metadata);
    if !blocker_assessment
        .blocking_dependency_declarations
        .is_empty()
        || !blocker_assessment.lifecycle_scripts.is_empty()
    {
        return DirectExecutionOutcome {
            decision: ClosureDecision::ExecutionRefused,
            reasons: blocker_assessment.reasons,
            command: ClosureCommandIdentity::from(intent),
            selected_bin: None,
            cwd: None,
            environment: fixture_environment(),
            exit_code: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
            detail: "package metadata contains unproven execution closure blockers".to_string(),
        };
    }
    if !artifact.metadata.dependency_declarations.is_empty() {
        return DirectExecutionOutcome {
            decision: ClosureDecision::ExecutionRefused,
            reasons: vec![M2Reason::UnsupportedClosure],
            command: ClosureCommandIdentity::from(intent),
            selected_bin: None,
            cwd: None,
            environment: fixture_environment(),
            exit_code: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
            detail: "package metadata contains dependency declarations outside M2 execution scope"
                .to_string(),
        };
    }

    let selected_bin = match select_package_bin(&artifact.metadata) {
        Ok(selected_bin) => selected_bin,
        Err(error) => {
            return DirectExecutionOutcome::refused(intent, error.reason, None, error.detail);
        }
    };
    let executable_identity = match identify_selected_bin(artifact, &selected_bin) {
        Ok(identity) => identity,
        Err(error) => {
            return DirectExecutionOutcome::refused(intent, error.reason, None, error.detail);
        }
    };

    let root = match canonicalize_fixture_path(&artifact.extraction_root) {
        Ok(root) => root,
        Err(detail) => {
            return DirectExecutionOutcome::refused(
                intent,
                M2Reason::UnsupportedClosure,
                Some(executable_identity),
                detail,
            );
        }
    };
    let executable = root.join(&selected_bin.relative_path);
    let environment = fixture_environment();
    let invocation = ProcessInvocation {
        executable,
        args: intent.forwarded_args.clone(),
        cwd: root.clone(),
        environment: environment.variables.clone(),
    };
    let output = crate::process_boundary::run_direct_process(&invocation);

    match output {
        Ok(output) => DirectExecutionOutcome {
            decision: ClosureDecision::Allow,
            reasons: Vec::new(),
            command: ClosureCommandIdentity::from(intent),
            selected_bin: Some(executable_identity),
            cwd: Some(root),
            environment,
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
            detail: "local fixture executed from verified root artifact bin".to_string(),
        },
        Err(error) => DirectExecutionOutcome::refused(
            intent,
            M2Reason::UnsupportedClosure,
            Some(executable_identity),
            format!("could not execute local fixture bin directly: {error}"),
        ),
    }
}

fn fixture_environment() -> DirectExecutionEnvironment {
    DirectExecutionEnvironment {
        inherited_environment_cleared: true,
        variables: BTreeMap::from([("SAFE_NPX_FIXTURE".to_string(), "1".to_string())]),
    }
}

fn canonicalize_fixture_path(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|error| format!("could not canonicalize fixture extraction root: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ArtifactIdentity, DependencyDeclarationKind, ExtractedDependencyDeclaration,
        ExtractedPackageMetadata, PackageSpec,
    };
    use std::fs;
    use std::io::Write;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::{AtomicU64, Ordering};
    #[cfg(unix)]
    #[test]
    fn executes_marked_local_fixture_and_records_evidence() {
        let fixture = TempRoot::new();
        fixture.mark_local();
        fixture.write_executable(
            "bin/create-example",
            b"#!/bin/sh\nprintf '%s|%s|%s' \"$SAFE_NPX_FIXTURE\" \"$1\" \"$2\"\n",
        );
        let artifact = artifact_with_metadata(
            fixture.path(),
            metadata_with_bins([("create-example", "bin/create-example")]),
        );
        let intent = intent_with_args(vec![
            "--template".to_string(),
            "react app".to_string(),
            "".to_string(),
        ]);

        let outcome = execute_local_fixture_package(&artifact, &intent);

        assert_eq!(outcome.decision, ClosureDecision::Allow);
        assert!(outcome.reasons.is_empty());
        assert_eq!(outcome.command.forwarded_args, intent.forwarded_args);
        assert!(outcome.selected_bin.is_some());
        assert_eq!(
            outcome
                .selected_bin
                .as_ref()
                .expect("selected bin evidence")
                .relative_path,
            "bin/create-example"
        );
        assert_eq!(
            outcome.environment.variables.get("SAFE_NPX_FIXTURE"),
            Some(&"1".to_string())
        );
        assert!(outcome.environment.inherited_environment_cleared);
        assert_eq!(outcome.exit_code, Some(0));
        assert_eq!(outcome.stdout, b"1|--template|react app");
        assert!(outcome.stderr.is_empty());
        assert_eq!(
            outcome.cwd,
            Some(
                fixture
                    .path()
                    .canonicalize()
                    .expect("fixture root should exist")
            )
        );
    }

    #[test]
    fn refuses_unmarked_roots_without_execution() {
        let fixture = TempRoot::new();
        fixture.write_file("bin/create-example", b"fixture");
        let artifact = artifact_with_metadata(
            fixture.path(),
            metadata_with_bins([("create-example", "bin/create-example")]),
        );

        let outcome = execute_local_fixture_package(&artifact, &intent_with_args(vec![]));

        assert_eq!(outcome.decision, ClosureDecision::ExecutionRefused);
        assert_eq!(outcome.reasons, vec![M2Reason::UnsupportedClosure]);
        assert!(outcome.selected_bin.is_none());
        assert!(outcome.exit_code.is_none());
    }

    #[test]
    fn refuses_lifecycle_scripts() {
        let fixture = marked_fixture_with_file("bin/create-example", b"fixture");
        let mut metadata = metadata_with_bins([("create-example", "bin/create-example")]);
        metadata
            .lifecycle_scripts
            .insert("postinstall".to_string(), "node postinstall.js".to_string());

        let outcome = execute_local_fixture_package(
            &artifact_with_metadata(fixture.path(), metadata),
            &intent_with_args(vec![]),
        );

        assert_eq!(outcome.decision, ClosureDecision::ExecutionRefused);
        assert_eq!(outcome.reasons, vec![M2Reason::LifecycleScriptPresent]);
        assert!(outcome.exit_code.is_none());
    }

    #[test]
    fn refuses_dependency_declarations() {
        assert_dependency_refused(DependencyDeclarationKind::Runtime);
    }

    #[test]
    fn refuses_dev_dependency_declarations() {
        assert_dependency_refused(DependencyDeclarationKind::Development);
    }

    fn assert_dependency_refused(kind: DependencyDeclarationKind) {
        let fixture = marked_fixture_with_file("bin/create-example", b"fixture");
        let mut metadata = metadata_with_bins([("create-example", "bin/create-example")]);
        metadata
            .dependency_declarations
            .push(ExtractedDependencyDeclaration {
                name: "left-pad".to_string(),
                requirement: "^6.0.0".to_string(),
                kind,
            });

        let outcome = execute_local_fixture_package(
            &artifact_with_metadata(fixture.path(), metadata),
            &intent_with_args(vec![]),
        );

        assert_eq!(outcome.decision, ClosureDecision::ExecutionRefused);
        assert_eq!(outcome.reasons, vec![M2Reason::UnsupportedClosure]);
        assert!(outcome.exit_code.is_none());
    }

    #[test]
    fn refuses_ambiguous_bins_as_execution_refused() {
        let fixture = marked_fixture_with_file("bin/create-example", b"fixture");
        let metadata = metadata_with_bins([
            ("create-example", "bin/create-example"),
            ("other", "bin/other"),
        ]);

        let outcome = execute_local_fixture_package(
            &artifact_with_metadata(fixture.path(), metadata),
            &intent_with_args(vec![]),
        );

        assert_eq!(outcome.decision, ClosureDecision::ExecutionRefused);
        assert_eq!(outcome.reasons, vec![M2Reason::AmbiguousBin]);
        assert!(outcome.exit_code.is_none());
    }

    #[test]
    fn refuses_missing_bins_as_execution_refused() {
        let fixture = TempRoot::new();
        fixture.mark_local();
        let metadata = metadata_with_bins([]);

        let outcome = execute_local_fixture_package(
            &artifact_with_metadata(fixture.path(), metadata),
            &intent_with_args(vec![]),
        );

        assert_eq!(outcome.decision, ClosureDecision::ExecutionRefused);
        assert_eq!(outcome.reasons, vec![M2Reason::MissingBin]);
        assert!(outcome.exit_code.is_none());
    }

    #[test]
    fn refuses_generated_shim_ambiguity() {
        let fixture = TempRoot::new();
        fixture.mark_local();
        let metadata = metadata_with_bins([("create-example", ".safe-npx/shims/create-example")]);

        let outcome = execute_local_fixture_package(
            &artifact_with_metadata(fixture.path(), metadata),
            &intent_with_args(vec![]),
        );

        assert_eq!(outcome.decision, ClosureDecision::ExecutionRefused);
        assert_eq!(outcome.reasons, vec![M2Reason::UnsupportedClosure]);
        assert!(outcome.exit_code.is_none());
        assert!(outcome.detail.contains("selected bin"));
    }

    fn marked_fixture_with_file(relative_path: &str, contents: &[u8]) -> TempRoot {
        let fixture = TempRoot::new();
        fixture.mark_local();
        fixture.write_file(relative_path, contents);
        fixture
    }

    fn intent_with_args(forwarded_args: Vec<String>) -> CommandIntent {
        CommandIntent::supported(
            PackageSpec::exact("create-example@1.2.3", "create-example", "1.2.3", None),
            forwarded_args,
        )
    }

    fn artifact_with_metadata(
        extraction_root: &Path,
        metadata: ExtractedPackageMetadata,
    ) -> ExtractedRootArtifact {
        ExtractedRootArtifact {
            artifact: ArtifactIdentity {
                name: "create-example".to_string(),
                version: "1.2.3".to_string(),
                integrity: "sha512-fixture".to_string(),
                digest_algorithm: "sha512".to_string(),
                digest: "fixture".to_string(),
            },
            extraction_root: extraction_root.to_path_buf(),
            metadata,
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
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "safe-npx-direct-{}-{}",
                std::process::id(),
                next_temp_id(),
            ));
            fs::create_dir_all(&path).expect("temp root should be creatable");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn mark_local(&self) {
            self.write_file(LOCAL_FIXTURE_MARKER, b"fixture\n");
        }

        fn write_file(&self, relative_path: &str, contents: &[u8]) {
            let path = self.path.join(relative_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("test parent should be creatable");
            }
            fs::write(path, contents).expect("test file should be writable");
        }

        #[cfg(unix)]
        fn write_executable(&self, relative_path: &str, contents: &[u8]) {
            let path = self.path.join(relative_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("test parent should be creatable");
            }
            let mut file = fs::File::create(&path).expect("test executable should be creatable");
            file.write_all(contents)
                .expect("test executable should be writable");
            let mut permissions = file
                .metadata()
                .expect("test executable metadata")
                .permissions();
            permissions.set_mode(0o700);
            fs::set_permissions(path, permissions).expect("test executable should be executable");
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn next_temp_id() -> u64 {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        NEXT_ID.fetch_add(1, Ordering::SeqCst)
    }
}
