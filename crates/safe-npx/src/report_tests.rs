//! Tests for M1 report rendering.

use crate::{
    build_report_with_resolver, render_report, run, Cli, Decision, M1Evidence, NpmMetadataClient,
    RegistryHttpResponse, RegistryTransport, RegistryTransportError, RootArtifactResolver,
    TarballDownloader, TarballHttpResponse, TarballTransport, TarballTransportError,
};
use base64::prelude::{Engine as _, BASE64_STANDARD};
use clap::Parser;
use sha2::{Digest, Sha512};
use std::cell::RefCell;

#[derive(Debug)]
struct StubRegistryTransport {
    responses: RefCell<Vec<Result<RegistryHttpResponse, RegistryTransportError>>>,
}

impl StubRegistryTransport {
    fn new(responses: Vec<Result<RegistryHttpResponse, RegistryTransportError>>) -> Self {
        Self {
            responses: RefCell::new(responses),
        }
    }
}

impl RegistryTransport for StubRegistryTransport {
    fn get(&self, _url: &str) -> Result<RegistryHttpResponse, RegistryTransportError> {
        self.responses.borrow_mut().remove(0)
    }
}

#[derive(Debug)]
struct StubTarballTransport {
    responses: RefCell<Vec<Result<TarballHttpResponse, TarballTransportError>>>,
}

impl StubTarballTransport {
    fn new(responses: Vec<Result<TarballHttpResponse, TarballTransportError>>) -> Self {
        Self {
            responses: RefCell::new(responses),
        }
    }
}

impl TarballTransport for StubTarballTransport {
    fn get_bytes(&self, _url: &str) -> Result<TarballHttpResponse, TarballTransportError> {
        self.responses.borrow_mut().remove(0)
    }
}

#[test]
fn parses_default_ask_decision() {
    let cli = Cli::parse_from(["safe-npx", "create-example@1.2.3"]);

    assert_eq!(cli.raw_package_spec(), "create-example@1.2.3");
    assert_eq!(cli.decision, Decision::Ask);
    assert!(!cli.json);
}

#[test]
fn builds_m1_verified_report() {
    let cli = Cli::parse_from(["safe-npx", "--decision", "deny", "create-example@1.2.3"]);
    let report = build_report_with_resolver(&cli, &verified_resolver(b"fixture-tarball"));

    assert_eq!(report.package_spec, "create-example@1.2.3");
    assert!(report.intent.is_supported());
    assert_eq!(report.recommendation, Decision::Deny);
    assert_eq!(report.status, "m1_evidence");
    assert!(report.note.contains("root artifact only"));
    assert!(matches!(
        report.m1,
        M1Evidence::Verified {
            integrity_status: "verified",
            ..
        }
    ));
}

#[test]
fn renders_json_for_agents() {
    let cli = Cli::parse_from(["safe-npx", "--json", "create-example@1.2.3"]);
    let output = run_with_resolver(&cli, &verified_resolver(b"fixture-tarball"));

    assert!(output.contains("\"package_spec\": \"create-example@1.2.3\""));
    assert!(output.contains("\"state\": \"supported\""));
    assert!(output.contains("\"name\": \"create-example\""));
    assert!(output.contains("\"version\": \"1.2.3\""));
    assert!(output.contains("\"recommendation\": \"ask\""));
    assert!(output.contains("\"state\": \"verified\""));
    assert!(output.contains("\"integrity_status\": \"verified\""));
    assert!(output.contains("\"digest_algorithm\": \"sha512\""));
}

#[test]
fn renders_json_with_forwarded_args_for_agents() {
    let cli = Cli::parse_from([
        "safe-npx",
        "--json",
        "create-example@1.2.3",
        "--",
        "--template",
        "react",
    ]);
    let output = run_with_resolver(&cli, &verified_resolver(b"fixture-tarball"));

    assert!(output.contains("\"forwarded_args\": ["));
    assert!(output.contains("\"--template\""));
    assert!(output.contains("\"react\""));
}

#[test]
fn renders_json_for_unsupported_specs() {
    let cli = Cli::parse_from(["safe-npx", "--json", "create-example@next"]);
    let output = run(&cli).expect("json rendering should succeed");

    assert!(output.contains("\"state\": \"unsupported\""));
    assert!(output.contains("\"reason\": \"unsupported_spec\""));
    assert!(output.contains("\"category\": \"version_range\""));
    assert!(output.contains("\"forwarded_args\": []"));
    assert!(output.contains("\"downloaded\": false"));
    assert!(!output.contains("\"execution\""));
}

#[test]
fn renders_json_for_malformed_specs() {
    let cli = Cli::parse_from(["safe-npx", "--json", "@scope/"]);
    let output = run(&cli).expect("json rendering should succeed");

    assert!(output.contains("\"state\": \"malformed\""));
    assert!(output.contains("\"reason\": \"malformed_spec\""));
    assert!(output.contains("\"raw\": \"@scope/\""));
    assert!(output.contains("\"downloaded\": false"));
    assert!(!output.contains("\"execution\""));
}

#[test]
fn renders_json_for_multi_token_unsupported_specs() {
    let cli = Cli::parse_from(["safe-npx", "--json", "npm", "exec", "create-example@1.2.3"]);
    let output = run(&cli).expect("json rendering should succeed");

    assert!(output.contains("\"package_spec\": \"npm exec create-example@1.2.3\""));
    assert!(output.contains("\"state\": \"unsupported\""));
    assert!(output.contains("\"category\": \"npm_exec_variant\""));
}

#[test]
fn renders_json_for_flagged_exec_variants() {
    let cases = [
        vec![
            "safe-npx",
            "--json",
            "npm",
            "exec",
            "--package",
            "create-example@1.2.3",
        ],
        vec![
            "safe-npx",
            "--json",
            "npm",
            "exec",
            "-c",
            "create-example@1.2.3 --help",
        ],
        vec!["safe-npx", "--json", "npx", "--yes", "create-example@1.2.3"],
    ];

    for case in cases {
        let cli = Cli::parse_from(case);
        let output = run(&cli).expect("json rendering should succeed");

        assert!(output.contains("\"state\": \"unsupported\""));
        assert!(output.contains("\"category\": \"npm_exec_variant\""));
        assert!(output.contains("\"downloaded\": false"));
    }
}

#[test]
fn renders_human_scaffold_output() {
    let cli = Cli::parse_from(["safe-npx", "--decision", "allow", "create-example@1.2.3"]);
    let output = run_with_resolver(&cli, &verified_resolver(b"fixture-tarball"));

    assert!(output.contains("Package: create-example@1.2.3"));
    assert!(output.contains("Parsed: create-example@1.2.3"));
    assert!(output.contains("Recommendation: Allow"));
    assert!(output.contains("M1 evidence: verified"));
    assert!(output.contains("Resolved: create-example@1.2.3"));
    assert!(output.contains("Integrity: verified"));
    assert!(output.contains("This Rust CLI does not execute package code in M1"));
}

#[test]
fn renders_human_refusal_output() {
    let cli = Cli::parse_from(["safe-npx", "create-example@next"]);
    let output = run(&cli).expect("text rendering should succeed");

    assert!(output.contains("Rejected: create-example@next"));
    assert!(output.contains("Reason: unsupported_spec"));
    assert!(output.contains("Category: version_range"));
    assert!(output.contains("Downloaded: false"));
}

#[test]
fn renders_json_for_integrity_mismatch() {
    let cli = Cli::parse_from(["safe-npx", "--json", "create-example@1.2.3"]);
    let resolver = resolver_with(
        metadata_body(&integrity_for(b"different-bytes")),
        b"fixture-tarball".to_vec(),
    );
    let output = run_with_resolver(&cli, &resolver);

    assert!(output.contains("\"recommendation\": \"deny\""));
    assert!(output.contains("\"state\": \"failed\""));
    assert!(output.contains("\"reason\": \"integrity_mismatch\""));
    assert!(output.contains("\"downloaded\": true"));
}

fn run_with_resolver(
    cli: &Cli,
    resolver: &RootArtifactResolver<StubRegistryTransport, StubTarballTransport>,
) -> String {
    let report = build_report_with_resolver(cli, resolver);
    render_report(cli, &report).expect("report rendering should succeed")
}

fn verified_resolver(
    bytes: &[u8],
) -> RootArtifactResolver<StubRegistryTransport, StubTarballTransport> {
    resolver_with(metadata_body(&integrity_for(bytes)), bytes.to_vec())
}

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

fn integrity_for(bytes: &[u8]) -> String {
    let digest = Sha512::digest(bytes);
    format!("sha512-{}", BASE64_STANDARD.encode(digest))
}
