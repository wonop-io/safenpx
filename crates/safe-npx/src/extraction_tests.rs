//! Tests for safe static root extraction.

use crate::{extract_verified_root_artifact, ArtifactIdentity, ExtractionErrorReason};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tar::{Builder, EntryType, Header};

#[test]
/// Verifies normal package extraction and metadata identity.
fn extracts_normal_package_metadata_tied_to_artifact_identity() {
    let workspace = TempRoot::new();
    let tarball = tgz_with_files(&[(
        "package/package.json",
        r#"{
            "name": "create-example",
            "version": "1.2.3",
            "bin": {"create-example": "bin/create.js"},
            "scripts": {"postinstall": "node postinstall.js", "test": "node test.js"},
            "dependencies": {"left-pad": "^1.3.0"}
        }"#,
    )]);
    let artifact = artifact_identity();

    let extracted = extract_verified_root_artifact(&tarball, artifact.clone(), workspace.path())
        .expect("normal package should extract");

    assert_eq!(extracted.artifact, artifact);
    assert_eq!(extracted.metadata.name.as_deref(), Some("create-example"));
    assert_eq!(extracted.metadata.version.as_deref(), Some("1.2.3"));
    assert_eq!(
        extracted
            .metadata
            .bins
            .get("create-example")
            .map(String::as_str),
        Some("bin/create.js")
    );
    assert_eq!(
        extracted
            .metadata
            .lifecycle_scripts
            .get("postinstall")
            .map(String::as_str),
        Some("node postinstall.js")
    );
    assert_eq!(extracted.metadata.lifecycle_scripts.len(), 1);
    assert_eq!(extracted.metadata.dependency_declarations.len(), 1);
    assert!(workspace.path().join("package/package.json").exists());
}

#[test]
/// Verifies traversal entries are rejected before writing outside root.
fn rejects_path_traversal_entries() {
    let workspace = TempRoot::new();
    let tarball = tgz_with_raw_file_path("../escape.txt", b"escape");

    let error = extract_verified_root_artifact(&tarball, artifact_identity(), workspace.path())
        .expect_err("traversal should fail");

    assert_eq!(error.reason, ExtractionErrorReason::UnsafePath);
    assert!(!workspace.path().join("../escape.txt").exists());
}

#[test]
/// Verifies symlink escapes are rejected.
fn rejects_symlink_escape_attempt() {
    let workspace = TempRoot::new();
    let tarball = tgz_with_symlink("package/link", "../escape");

    let error = extract_verified_root_artifact(&tarball, artifact_identity(), workspace.path())
        .expect_err("symlink escape should fail");

    assert_eq!(error.reason, ExtractionErrorReason::UnsafeLink);
}

#[test]
/// Verifies missing package metadata fails closed.
fn rejects_missing_package_json() {
    let workspace = TempRoot::new();
    let tarball = tgz_with_files(&[("package/README.md", "readme")]);

    let error = extract_verified_root_artifact(&tarball, artifact_identity(), workspace.path())
        .expect_err("missing package.json should fail");

    assert_eq!(error.reason, ExtractionErrorReason::MissingPackageJson);
}

#[test]
/// Verifies malformed package metadata fails closed.
fn rejects_malformed_package_json() {
    let workspace = TempRoot::new();
    let tarball = tgz_with_files(&[("package/package.json", "{not-json")]);

    let error = extract_verified_root_artifact(&tarball, artifact_identity(), workspace.path())
        .expect_err("malformed package.json should fail");

    assert_eq!(error.reason, ExtractionErrorReason::MalformedPackageJson);
}

#[test]
/// Verifies extraction roots must not contain pre-existing filesystem state.
fn rejects_non_empty_extraction_root() {
    let workspace = TempRoot::new();
    fs::write(workspace.path().join("preexisting"), b"state")
        .expect("preexisting file should be writable");
    let tarball = tgz_with_files(&[("package/package.json", "{}")]);

    let error = extract_verified_root_artifact(&tarball, artifact_identity(), workspace.path())
        .expect_err("non-empty root should fail");

    assert_eq!(error.reason, ExtractionErrorReason::NonEmptyExtractionRoot);
}

#[test]
/// Verifies stale metadata in a reused root cannot be tied to a new artifact.
fn rejects_stale_metadata_in_reused_root() {
    let workspace = TempRoot::new();
    fs::create_dir_all(workspace.path().join("package")).expect("package dir should be writable");
    fs::write(
        workspace.path().join("package/package.json"),
        br#"{"name":"stale","version":"0.0.1"}"#,
    )
    .expect("stale package.json should be writable");
    let tarball = tgz_with_files(&[("package/README.md", "readme")]);

    let error = extract_verified_root_artifact(&tarball, artifact_identity(), workspace.path())
        .expect_err("stale metadata root should fail");

    assert_eq!(error.reason, ExtractionErrorReason::NonEmptyExtractionRoot);
}

#[test]
/// Verifies string bin metadata uses the unscoped package name.
fn string_bin_uses_unscoped_package_name() {
    let workspace = TempRoot::new();
    let tarball = tgz_with_files(&[(
        "package/package.json",
        r#"{"name":"@scope/create-example","version":"1.2.3","bin":"cli.js"}"#,
    )]);

    let extracted =
        extract_verified_root_artifact(&tarball, scoped_artifact_identity(), workspace.path())
            .expect("scoped package should extract");

    assert_eq!(
        extracted
            .metadata
            .bins
            .get("create-example")
            .map(String::as_str),
        Some("cli.js")
    );
}

/// Build a gzip-compressed tarball with regular file entries.
fn tgz_with_files(files: &[(&str, &str)]) -> Vec<u8> {
    let mut tarball = Vec::new();
    {
        let encoder = GzEncoder::new(&mut tarball, Compression::default());
        let mut builder = Builder::new(encoder);
        for (path, contents) in files {
            append_file(&mut builder, path, contents.as_bytes());
        }
        builder.finish().expect("tar builder should finish");
    }
    tarball
}

/// Build a gzip-compressed tarball with a raw path the safe builder rejects.
fn tgz_with_raw_file_path(path: &str, contents: &[u8]) -> Vec<u8> {
    let mut tarball = Vec::new();
    {
        let mut encoder = GzEncoder::new(&mut tarball, Compression::default());
        encoder
            .write_all(&raw_tar_file(path, contents))
            .expect("raw tar should gzip");
        encoder.finish().expect("gzip encoder should finish");
    }
    tarball
}

/// Build a minimal tar archive containing one regular file.
fn raw_tar_file(path: &str, contents: &[u8]) -> Vec<u8> {
    let mut archive = Vec::new();
    let mut header = [0_u8; 512];

    write_bytes(&mut header[0..100], path.as_bytes());
    write_octal(&mut header[100..108], 0o644);
    write_octal(&mut header[108..116], 0);
    write_octal(&mut header[116..124], 0);
    write_octal(&mut header[124..136], contents.len() as u64);
    write_octal(&mut header[136..148], 0);
    header[148..156].fill(b' ');
    header[156] = b'0';
    write_bytes(&mut header[257..263], b"ustar\0");
    write_bytes(&mut header[263..265], b"00");
    let checksum = header.iter().map(|byte| *byte as u32).sum::<u32>();
    write_octal(&mut header[148..156], checksum as u64);

    archive.extend_from_slice(&header);
    archive.extend_from_slice(contents);
    let padding = (512 - (contents.len() % 512)) % 512;
    archive.extend(std::iter::repeat_n(0, padding));
    archive.extend_from_slice(&[0_u8; 1024]);
    archive
}

/// Build a gzip-compressed tarball with one symlink entry.
fn tgz_with_symlink(path: &str, target: &str) -> Vec<u8> {
    let mut tarball = Vec::new();
    {
        let encoder = GzEncoder::new(&mut tarball, Compression::default());
        let mut builder = Builder::new(encoder);
        let mut header = Header::new_gnu();
        header.set_entry_type(EntryType::Symlink);
        header.set_size(0);
        header.set_cksum();
        builder
            .append_link(&mut header, path, target)
            .expect("symlink should append");
        builder.finish().expect("tar builder should finish");
    }
    tarball
}

/// Append a regular file to a tar builder.
fn append_file(builder: &mut Builder<GzEncoder<&mut Vec<u8>>>, path: &str, contents: &[u8]) {
    let mut header = Header::new_gnu();
    header.set_size(contents.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, path, contents)
        .expect("file should append");
}

/// Write bytes into a fixed-width tar header field.
fn write_bytes(field: &mut [u8], value: &[u8]) {
    field[..value.len()].copy_from_slice(value);
}

/// Write a right-aligned octal tar header field.
fn write_octal(field: &mut [u8], value: u64) {
    let encoded = format!("{value:0width$o}\0", width = field.len() - 1);
    field.copy_from_slice(encoded.as_bytes());
}

/// Return fixture artifact identity.
fn artifact_identity() -> ArtifactIdentity {
    ArtifactIdentity {
        name: "create-example".to_string(),
        version: "1.2.3".to_string(),
        integrity: "sha512-fixture".to_string(),
        digest_algorithm: "sha512".to_string(),
        digest: "abc123".to_string(),
    }
}

/// Return scoped fixture artifact identity.
fn scoped_artifact_identity() -> ArtifactIdentity {
    ArtifactIdentity {
        name: "@scope/create-example".to_string(),
        version: "1.2.3".to_string(),
        integrity: "sha512-fixture".to_string(),
        digest_algorithm: "sha512".to_string(),
        digest: "abc123".to_string(),
    }
}

/// Temporary extraction root for tests.
struct TempRoot {
    path: PathBuf,
}

impl TempRoot {
    /// Create a unique temporary extraction root.
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "safe-npx-extract-{}-{nanos}-{}",
            std::process::id(),
            next_temp_id()
        ));
        fs::create_dir_all(&path).expect("temp root should be creatable");

        Self { path }
    }

    /// Return the temporary extraction root path.
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempRoot {
    /// Remove the temporary extraction root.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Return a unique id for parallel test temp roots.
fn next_temp_id() -> u64 {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}
