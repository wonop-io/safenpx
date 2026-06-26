//! Canary carryover tests for the M3 inspect-mode pipeline.

use crate::{
    build_report_with_resolver, canary_fixtures, render_report, CanaryFixture, CanaryTrapKind, Cli,
    M1Evidence, NpmMetadataClient, RegistryHttpResponse, RegistryTransport, RegistryTransportError,
    RootArtifactResolver, TarballDownloader, TarballHttpResponse, TarballTransport,
    TarballTransportError,
};
use base64::prelude::{Engine as _, BASE64_STANDARD};
use clap::Parser;
use flate2::{write::GzEncoder, Compression};
use sha2::{Digest, Sha512};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tar::{Builder, Header};

#[derive(Debug)]
/// Registry transport fixture that returns scripted metadata responses.
struct StubRegistryTransport {
    responses: RefCell<Vec<Result<RegistryHttpResponse, RegistryTransportError>>>,
}

impl StubRegistryTransport {
    /// Build a registry transport fixture from ordered responses.
    fn new(responses: Vec<Result<RegistryHttpResponse, RegistryTransportError>>) -> Self {
        Self {
            responses: RefCell::new(responses),
        }
    }
}

impl RegistryTransport for StubRegistryTransport {
    /// Return the next scripted registry response.
    fn get(&self, _url: &str) -> Result<RegistryHttpResponse, RegistryTransportError> {
        self.responses.borrow_mut().remove(0)
    }
}

#[derive(Debug)]
/// Tarball transport fixture that returns scripted artifact responses.
struct StubTarballTransport {
    responses: RefCell<Vec<Result<TarballHttpResponse, TarballTransportError>>>,
}

impl StubTarballTransport {
    /// Build a tarball transport fixture from ordered responses.
    fn new(responses: Vec<Result<TarballHttpResponse, TarballTransportError>>) -> Self {
        Self {
            responses: RefCell::new(responses),
        }
    }
}

impl TarballTransport for StubTarballTransport {
    /// Return the next scripted tarball response.
    fn get_bytes(&self, _url: &str) -> Result<TarballHttpResponse, TarballTransportError> {
        self.responses.borrow_mut().remove(0)
    }
}

#[test]
/// Human inspect rendering leaves every package-code canary sentinel absent.
fn inspect_human_output_leaves_canary_sentinels_absent() {
    assert_inspect_output_leaves_canary_sentinels_absent(RenderMode::Human);
}

#[test]
/// JSON inspect rendering leaves every package-code canary sentinel absent.
fn inspect_json_output_leaves_canary_sentinels_absent() {
    assert_inspect_output_leaves_canary_sentinels_absent(RenderMode::Json);
}

#[test]
/// Inspect failure rendering leaves every package-code canary sentinel absent.
fn inspect_failure_output_leaves_canary_sentinels_absent() {
    assert_inspect_failure_leaves_canary_sentinels_absent(RenderMode::Human);
    assert_inspect_failure_leaves_canary_sentinels_absent(RenderMode::Json);
}

/// Render mode exercised by canary carryover tests.
#[derive(Clone, Copy, Debug)]
enum RenderMode {
    /// Human terminal rendering.
    Human,
    /// JSON rendering for agents and CI.
    Json,
}

impl RenderMode {
    /// Return CLI arguments for this render mode.
    fn cli_args(self) -> Vec<&'static str> {
        match self {
            RenderMode::Human => vec!["safe-npx", "inspect", "create-example@1.2.3"],
            RenderMode::Json => vec!["safe-npx", "--json", "inspect", "create-example@1.2.3"],
        }
    }
}

/// Assert the real inspect pipeline does not trip any bundled canary fixture.
fn assert_inspect_output_leaves_canary_sentinels_absent(render_mode: RenderMode) {
    let workspace = CanaryTempRoot::new();

    for fixture in canary_fixtures() {
        let sentinel = fixture.sentinel_path(workspace.path());
        let tarball = canary_package_tarball(&fixture, &sentinel);
        let cli = Cli::parse_from(render_mode.cli_args());
        let report = build_report_with_resolver(&cli, &verified_resolver(&tarball));
        let output = render_report(&cli, &report).expect("inspect report should render");

        assert!(
            matches!(report.m1, M1Evidence::Verified { .. }),
            "{} {:?} should inspect successfully before execution checks",
            fixture.id,
            fixture.trap_kind
        );
        assert!(
            !sentinel.exists(),
            "{} {:?} created sentinel {:?} during {:?} inspect output",
            fixture.id,
            fixture.trap_kind,
            sentinel,
            render_mode
        );
        assert!(
            output.contains("m3_inspect"),
            "{} {:?} should use the explicit inspect pipeline",
            fixture.id,
            fixture.trap_kind
        );
    }
}

/// Assert inspect failures do not trip any bundled canary fixture.
fn assert_inspect_failure_leaves_canary_sentinels_absent(render_mode: RenderMode) {
    let workspace = CanaryTempRoot::new();

    for fixture in canary_fixtures() {
        let sentinel = fixture.sentinel_path(workspace.path());
        let tarball = canary_package_tarball(&fixture, &sentinel);
        let cli = Cli::parse_from(render_mode.cli_args());
        let report =
            build_report_with_resolver(&cli, &resolver_with_integrity(&tarball, "sha512-wrong"));
        let output = render_report(&cli, &report).expect("inspect failure should render");

        assert!(
            matches!(report.m1, M1Evidence::Failed { .. }),
            "{} {:?} should fail before any package-code path",
            fixture.id,
            fixture.trap_kind
        );
        assert!(
            !sentinel.exists(),
            "{} {:?} created sentinel {:?} during {:?} inspect failure",
            fixture.id,
            fixture.trap_kind,
            sentinel,
            render_mode
        );
        assert!(
            output.contains("integrity_mismatch") || output.contains("IntegrityMismatch"),
            "{} {:?} should render the integrity failure",
            fixture.id,
            fixture.trap_kind
        );
    }
}

/// Build a resolver that serves verified canary tarball bytes.
fn verified_resolver(
    bytes: &[u8],
) -> RootArtifactResolver<StubRegistryTransport, StubTarballTransport> {
    resolver_with_integrity(bytes, &integrity_for(bytes))
}

/// Build a resolver that serves canary bytes with explicit integrity metadata.
fn resolver_with_integrity(
    bytes: &[u8],
    integrity: &str,
) -> RootArtifactResolver<StubRegistryTransport, StubTarballTransport> {
    RootArtifactResolver::new(
        NpmMetadataClient::public(StubRegistryTransport::new(vec![Ok(RegistryHttpResponse {
            status: 200,
            body: metadata_body(integrity),
        })])),
        TarballDownloader::new(StubTarballTransport::new(vec![Ok(TarballHttpResponse {
            status: 200,
            bytes: bytes.to_vec(),
        })])),
    )
}

/// Return minimal npm metadata for a verified canary tarball.
fn metadata_body(integrity: &str) -> String {
    format!(
        r#"{{
            "versions": {{
                "1.2.3": {{
                    "version": "1.2.3",
                    "dist": {{
                        "tarball": "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz",
                        "integrity": "{}"
                    }}
                }}
            }}
        }}"#,
        integrity
    )
}

/// Return the npm sha512 integrity string for fixture bytes.
fn integrity_for(bytes: &[u8]) -> String {
    let digest = Sha512::digest(bytes);
    format!("sha512-{}", BASE64_STANDARD.encode(digest))
}

/// Build a local package tarball that contains one canary trap surface.
fn canary_package_tarball(fixture: &CanaryFixture, sentinel: &Path) -> Vec<u8> {
    let mut gzip = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = Builder::new(&mut gzip);
        append_file(
            &mut builder,
            "package/package.json",
            package_json_for(fixture).as_bytes(),
        );
        append_file(
            &mut builder,
            trap_file_path(fixture),
            trap_payload(sentinel).as_bytes(),
        );
        if fixture.trap_kind == CanaryTrapKind::DependencyLifecycle {
            append_file(
                &mut builder,
                "package/node_modules/canary-dep/package.json",
                dependency_package_json().as_bytes(),
            );
        }
        builder.finish().expect("tar builder should finish");
    }
    gzip.finish().expect("gzip encoder should finish")
}

/// Return package metadata that would execute a trap if install or bin ran.
fn package_json_for(fixture: &CanaryFixture) -> String {
    match fixture.trap_kind {
        CanaryTrapKind::RootBinary => {
            r#"{"name":"create-example","version":"1.2.3","bin":{"create-example":"bin/root-binary.js"}}"#.to_string()
        }
        CanaryTrapKind::RootPreinstall => {
            r#"{"name":"create-example","version":"1.2.3","scripts":{"preinstall":"node traps/root-preinstall.js"}}"#.to_string()
        }
        CanaryTrapKind::RootInstall => {
            r#"{"name":"create-example","version":"1.2.3","scripts":{"install":"node traps/root-install.js"}}"#.to_string()
        }
        CanaryTrapKind::RootPostinstall => {
            r#"{"name":"create-example","version":"1.2.3","scripts":{"postinstall":"node traps/root-postinstall.js"}}"#.to_string()
        }
        CanaryTrapKind::DependencyLifecycle => {
            r#"{"name":"create-example","version":"1.2.3","dependencies":{"canary-dep":"1.0.0"}}"#.to_string()
        }
        CanaryTrapKind::GeneratedShim => {
            r#"{"name":"create-example","version":"1.2.3","bin":{"create-example":"node_modules/.bin/create-example"}}"#.to_string()
        }
        CanaryTrapKind::NetworkAttempt => {
            r#"{"name":"create-example","version":"1.2.3","scripts":{"postinstall":"node traps/network-attempt.js"}}"#.to_string()
        }
    }
}

/// Return dependency metadata that would execute a lifecycle trap if installed.
fn dependency_package_json() -> String {
    r#"{"name":"canary-dep","version":"1.0.0","scripts":{"postinstall":"node postinstall.js"}}"#
        .to_string()
}

/// Return the trap payload path inside the package tarball.
fn trap_file_path(fixture: &CanaryFixture) -> &'static str {
    match fixture.trap_kind {
        CanaryTrapKind::RootBinary => "package/bin/root-binary.js",
        CanaryTrapKind::RootPreinstall => "package/traps/root-preinstall.js",
        CanaryTrapKind::RootInstall => "package/traps/root-install.js",
        CanaryTrapKind::RootPostinstall => "package/traps/root-postinstall.js",
        CanaryTrapKind::DependencyLifecycle => "package/node_modules/canary-dep/postinstall.js",
        CanaryTrapKind::GeneratedShim => "package/node_modules/.bin/create-example",
        CanaryTrapKind::NetworkAttempt => "package/traps/network-attempt.js",
    }
}

/// Return JavaScript that would create the sentinel if package code ran.
fn trap_payload(sentinel: &Path) -> String {
    let sentinel_json =
        serde_json::to_string(&sentinel.display().to_string()).expect("path should serialize");
    format!("require('fs').writeFileSync({sentinel_json}, 'trap ran');\n")
}

/// Append a fixture file to the package tarball.
fn append_file(builder: &mut Builder<&mut GzEncoder<Vec<u8>>>, path: &str, bytes: &[u8]) {
    let mut header = Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    builder
        .append_data(&mut header, path, bytes)
        .expect("fixture file should append");
}

/// Temporary sentinel root for M3 canary tests.
struct CanaryTempRoot {
    path: PathBuf,
}

impl CanaryTempRoot {
    /// Create a unique sentinel root for this test.
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "safe-npx-m3-canary-{}-{nanos}-{}",
            std::process::id(),
            next_temp_id()
        ));
        fs::create_dir_all(&path).expect("canary temp root should be creatable");

        Self { path }
    }

    /// Return the temporary sentinel root path.
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for CanaryTempRoot {
    /// Remove the temporary sentinel root.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Return a unique id for parallel test temp roots.
fn next_temp_id() -> u64 {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}
