//! Tests for the M3 inspect-mode pipeline.

use crate::report::exit_code_for_report;
use crate::{
    build_report_with_resolver, render_report, Cli, M1Evidence, NpmMetadataClient,
    RegistryHttpResponse, RegistryTransport, RegistryTransportError, RootArtifactResolver,
    TarballDownloader, TarballHttpResponse, TarballTransport, TarballTransportError,
};
use base64::prelude::{Engine as _, BASE64_STANDARD};
use flate2::{write::GzEncoder, Compression};
use sha2::{Digest, Sha512};
use std::cell::RefCell;
use tar::{Builder, Header};

#[derive(Debug)]
/// Registry transport fixture that returns scripted metadata responses.
struct StubRegistryTransport {
    responses: RefCell<Vec<Result<RegistryHttpResponse, RegistryTransportError>>>,
}

impl StubRegistryTransport {
    /// Build a registry fixture from ordered responses.
    fn new(responses: Vec<Result<RegistryHttpResponse, RegistryTransportError>>) -> Self {
        Self {
            responses: RefCell::new(responses),
        }
    }
}

impl RegistryTransport for StubRegistryTransport {
    /// Return the next scripted metadata response.
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
    /// Build a tarball fixture from ordered responses.
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
/// Inspect is parsed as a first-class action token.
fn parses_explicit_inspect_action() {
    let cli = Cli::parse_from([
        "safe-npx",
        "inspect",
        "create-example@1.2.3",
        "--",
        "--template",
        "react",
    ]);

    assert!(cli.is_inspect_action());
    assert_eq!(cli.raw_package_spec(), "create-example@1.2.3");
    assert_eq!(cli.forwarded_args(), ["--template", "react"]);
}

#[test]
/// Inspect mode extracts package metadata without moving into execution.
fn inspect_action_extracts_static_package_metadata_and_stops() {
    let tarball = package_tarball(
        r#"{
            "name":"create-example",
            "version":"1.2.3",
            "bin":{"create-example":"bin/create.js"}
        }"#,
    );
    let cli = Cli::parse_from(["safe-npx", "inspect", "create-example@1.2.3"]);
    let report = build_report_with_resolver(&cli, &verified_resolver(&tarball));

    assert_eq!(report.status, "m3_inspect");
    assert!(report.note.contains("statically extracts"));
    let M1Evidence::Verified {
        static_extraction: Some(static_extraction),
        ..
    } = report.m1
    else {
        panic!("inspect action should extract static metadata");
    };
    assert_eq!(static_extraction.status, "extracted");
    assert_eq!(
        static_extraction.metadata.bins.get("create-example"),
        Some(&"bin/create.js".to_string())
    );
    assert!(static_extraction.metadata.lifecycle_scripts.is_empty());
    assert!(static_extraction
        .metadata
        .dependency_declarations
        .is_empty());
}

#[test]
/// Invalid tarballs are reported as inspection failures instead of panicking.
fn inspect_action_reports_extraction_failure_without_panic() {
    let cli = Cli::parse_from(["safe-npx", "inspect", "create-example@1.2.3"]);
    let report = build_report_with_resolver(&cli, &verified_resolver(b"not-a-tarball"));

    assert_eq!(exit_code_for_report(&report), 3);
    let M1Evidence::Failed {
        reason,
        downloaded,
        detail,
    } = report.m1
    else {
        panic!("invalid archive should be reported as inspect failure");
    };
    assert_eq!(reason, crate::M1Reason::RegistryError);
    assert!(downloaded);
    assert!(detail.contains("static extraction failed"));
}

#[test]
/// Integrity mismatches stay in the existing failure vocabulary.
fn inspect_action_reports_integrity_failure_without_extraction() {
    let tarball = package_tarball(r#"{"name":"create-example","version":"1.2.3"}"#);
    let wrong_integrity = integrity_for(b"different-bytes");
    let resolver = resolver_with(metadata_body(&wrong_integrity), tarball);
    let cli = Cli::parse_from(["safe-npx", "inspect", "create-example@1.2.3"]);
    let report = build_report_with_resolver(&cli, &resolver);

    assert_eq!(exit_code_for_report(&report), 4);
    let M1Evidence::Failed {
        reason,
        downloaded,
        detail: _,
    } = report.m1
    else {
        panic!("integrity mismatch should be reported as inspect failure");
    };
    assert_eq!(reason, crate::M1Reason::IntegrityMismatch);
    assert!(downloaded);
}

#[test]
/// Lifecycle scripts and dependency declarations appear as static evidence.
fn inspect_action_reports_m2_closure_blockers_as_static_metadata() {
    let tarball = package_tarball(
        r#"{
            "name":"create-example",
            "version":"1.2.3",
            "scripts":{"postinstall":"node postinstall.js"},
            "dependencies":{"left-pad":"1.3.0"}
        }"#,
    );
    let cli = Cli::parse_from(["safe-npx", "--json", "inspect", "create-example@1.2.3"]);
    let output = run_with_resolver(&cli, &verified_resolver(&tarball));

    assert!(output.contains("\"static_extraction\""));
    assert!(output.contains("\"lifecycle_scripts\""));
    assert!(output.contains("\"postinstall\""));
    assert!(output.contains("\"dependency_declarations\""));
    assert!(output.contains("\"left-pad\""));
    assert!(output.contains("\"execution\": null"));
}

#[test]
/// Human inspect output includes root package facts, not only counts.
fn inspect_human_output_reports_root_package_facts() {
    let tarball = package_tarball(
        r#"{
            "name":"create-example",
            "version":"1.2.3",
            "bin":{"create-example":"bin/create.js","helper":"bin/helper.js"},
            "scripts":{"postinstall":"node postinstall.js"},
            "dependencies":{"left-pad":"1.3.0"}
        }"#,
    );
    let cli = Cli::parse_from(["safe-npx", "inspect", "create-example@1.2.3"]);
    let output = run_with_resolver(&cli, &verified_resolver(&tarball));

    assert!(output.contains("create-example -> bin/create.js"));
    assert!(output.contains("helper -> bin/helper.js"));
    assert!(output.contains("postinstall -> node postinstall.js"));
    assert!(output.contains("left-pad (Runtime) 1.3.0 [declaration_only]"));
    assert!(output.contains("Recommendation: Ask"));
}

#[test]
/// Unsupported inspect specs stop before any network download.
fn inspect_action_keeps_unsupported_specs_before_downloads() {
    let cli = Cli::parse_from(["safe-npx", "inspect", "create-example@latest"]);
    let report = build_report_with_resolver(&cli, &verified_resolver(b"unused"));
    let output = render_report(&cli, &report).expect("unsupported inspect should render");

    assert!(output.contains("Rejected: create-example@latest"));
    assert!(output.contains("Reason: unsupported_spec"));
    assert!(output.contains("Downloaded: false"));
    assert_eq!(exit_code_for_report(&report), 2);
}

/// Render a report through the supplied resolver.
fn run_with_resolver(
    cli: &Cli,
    resolver: &RootArtifactResolver<StubRegistryTransport, StubTarballTransport>,
) -> String {
    let report = build_report_with_resolver(cli, resolver);
    render_report(cli, &report).expect("report rendering should succeed")
}

/// Build a resolver that serves verified bytes for the fixture package.
fn verified_resolver(
    bytes: &[u8],
) -> RootArtifactResolver<StubRegistryTransport, StubTarballTransport> {
    resolver_with(metadata_body(&integrity_for(bytes)), bytes.to_vec())
}

/// Build a resolver from explicit registry metadata and tarball bytes.
fn resolver_with(
    metadata_body: String,
    tarball_bytes: Vec<u8>,
) -> RootArtifactResolver<StubRegistryTransport, StubTarballTransport> {
    RootArtifactResolver::new(
        NpmMetadataClient::public(StubRegistryTransport::new(vec![Ok(RegistryHttpResponse {
            status: 200,
            body: metadata_body,
        })])),
        TarballDownloader::new(StubTarballTransport::new(vec![Ok(TarballHttpResponse {
            status: 200,
            bytes: tarball_bytes,
        })])),
    )
}

/// Return minimal npm metadata for the fixture package.
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

/// Build a gzipped npm package tarball containing the supplied package JSON.
fn package_tarball(package_json: &str) -> Vec<u8> {
    let mut gzip = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = Builder::new(&mut gzip);
        append_file(
            &mut builder,
            "package/package.json",
            package_json.as_bytes(),
        );
        append_file(
            &mut builder,
            "package/bin/create.js",
            b"#!/usr/bin/env node\n",
        );
        builder.finish().expect("tar builder should finish");
    }
    gzip.finish().expect("gzip encoder should finish")
}

/// Append a file to a fixture tarball.
fn append_file(builder: &mut Builder<&mut GzEncoder<Vec<u8>>>, path: &str, bytes: &[u8]) {
    let mut header = Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, path, bytes)
        .expect("fixture file should append");
}
