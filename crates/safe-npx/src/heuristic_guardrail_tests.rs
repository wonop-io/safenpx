//! Guardrails ensuring inspect heuristics stay report-only in M4.

use crate::{
    build_report_with_resolver, render_report, Cli, InspectHeuristicKind, NpmMetadataClient,
    RegistryHttpResponse, RegistryTransport, RegistryTransportError, RootArtifactResolver,
    TarballDownloader, TarballHttpResponse, TarballTransport, TarballTransportError,
};
use base64::prelude::{Engine as _, BASE64_STANDARD};
use flate2::{write::GzEncoder, Compression};
use sha2::{Digest, Sha512};
use std::cell::RefCell;
use tar::{Builder, Header};

#[derive(Debug)]
struct StubRegistryTransport {
    responses: RefCell<Vec<Result<RegistryHttpResponse, RegistryTransportError>>>,
}

impl StubRegistryTransport {
    fn ok(body: String) -> Self {
        Self {
            responses: RefCell::new(vec![Ok(RegistryHttpResponse { status: 200, body })]),
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
    fn ok(bytes: Vec<u8>) -> Self {
        Self {
            responses: RefCell::new(vec![Ok(TarballHttpResponse { status: 200, bytes })]),
        }
    }
}

impl TarballTransport for StubTarballTransport {
    fn get_bytes(&self, _url: &str) -> Result<TarballHttpResponse, TarballTransportError> {
        self.responses.borrow_mut().remove(0)
    }
}

#[test]
fn unusual_shape_heuristic_only_does_not_deny() {
    let tarball = package_tarball(r#"{"name":"create-example","version":"1.2.3"}"#);
    let cli = Cli::parse_from([
        "safe-npx",
        "--decision",
        "allow",
        "inspect",
        "create-example@1.2.3",
    ]);
    let report = build_report_with_resolver(&cli, &resolver_with(tarball));
    let human = render_report(&cli, &report).expect("human report should render");

    assert_eq!(
        report.inspect.decision.recommendation,
        crate::Decision::Allow
    );
    assert_eq!(
        report.inspect.decision.reasons,
        vec!["caller_requested_allow"]
    );
    assert!(report
        .inspect
        .heuristics
        .iter()
        .all(|signal| signal.report_only));
    assert!(report
        .inspect
        .heuristics
        .iter()
        .any(|signal| signal.kind == InspectHeuristicKind::UnusualPackageShape));
    assert!(human.contains("[Heuristics: provisional risk signals]"));
    assert!(human.contains("- unusual_package_shape [report_only]"));
    assert!(!human.contains("Decision reasons: deny"));
}

#[test]
fn heuristic_only_json_does_not_promote_to_policy_reason() {
    let tarball = package_tarball(r#"{"name":"create-example","version":"1.2.3"}"#);
    let cli = Cli::parse_from([
        "safe-npx",
        "--json",
        "--decision",
        "allow",
        "inspect",
        "create-example@1.2.3",
    ]);
    let report = build_report_with_resolver(&cli, &resolver_with(tarball));
    let json = render_report(&cli, &report).expect("json report should render");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["decision"], "allow");
    assert_eq!(value["required_next_action"], "none");
    assert_eq!(
        value["reasons"],
        serde_json::json!(["caller_requested_allow"])
    );
    assert_eq!(
        value["policy"]["rule_ids"],
        serde_json::json!(["caller_recommendation"])
    );
    assert!(value["policy"]["findings"].as_array().unwrap().is_empty());
    assert_eq!(value["heuristics"][0]["kind"], "unusual_package_shape");
    assert_eq!(value["heuristics"][0]["report_only"], true);
}

#[test]
fn dependency_heuristic_only_json_does_not_promote_to_policy_reason() {
    let tarball = package_tarball(
        r#"{
            "name":"create-example",
            "version":"1.2.3",
            "bin":{"create-example":"bin/create.js"},
            "dependencies":{"left-pad":"1.3.0"}
        }"#,
    );
    let cli = Cli::parse_from([
        "safe-npx",
        "--json",
        "--decision",
        "allow",
        "inspect",
        "create-example@1.2.3",
    ]);
    let report = build_report_with_resolver(&cli, &resolver_with(tarball));
    let json = render_report(&cli, &report).expect("json report should render");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["decision"], "allow");
    assert_eq!(value["required_next_action"], "none");
    assert_eq!(
        value["reasons"],
        serde_json::json!(["caller_requested_allow"])
    );
    assert!(value["policy"]["findings"].as_array().unwrap().is_empty());
    assert_eq!(
        value["heuristics"][0]["kind"],
        "dependency_declarations_present"
    );
    assert_eq!(value["heuristics"][0]["report_only"], true);
}

#[test]
fn lifecycle_heuristic_asks_but_does_not_deny() {
    let tarball = package_tarball(
        r#"{
            "name":"create-example",
            "version":"1.2.3",
            "bin":{"create-example":"bin/create.js"},
            "scripts":{"postinstall":"node postinstall.js"}
        }"#,
    );
    let cli = Cli::parse_from([
        "safe-npx",
        "--decision",
        "allow",
        "inspect",
        "create-example@1.2.3",
    ]);
    let report = build_report_with_resolver(&cli, &resolver_with(tarball));

    assert_eq!(report.inspect.decision.recommendation, crate::Decision::Ask);
    assert!(report
        .inspect
        .decision
        .reasons
        .contains(&"lifecycle_script_present".to_string()));
    assert!(report
        .inspect
        .heuristics
        .iter()
        .all(|signal| signal.report_only));
    assert!(report
        .inspect
        .heuristics
        .iter()
        .any(|signal| signal.kind == InspectHeuristicKind::LifecycleScriptsPresent));
}

fn resolver_with(
    tarball_bytes: Vec<u8>,
) -> RootArtifactResolver<StubRegistryTransport, StubTarballTransport> {
    let integrity = integrity_for(&tarball_bytes);
    RootArtifactResolver::new(
        NpmMetadataClient::public(StubRegistryTransport::ok(metadata_body(&integrity))),
        TarballDownloader::new(StubTarballTransport::ok(tarball_bytes)),
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

fn package_tarball(package_json: &str) -> Vec<u8> {
    let mut tarball = Vec::new();
    {
        let encoder = GzEncoder::new(&mut tarball, Compression::default());
        let mut builder = Builder::new(encoder);
        let mut header = Header::new_gnu();
        header.set_size(package_json.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "package/package.json", package_json.as_bytes())
            .expect("package.json should append");
        builder.finish().expect("tar builder should finish");
    }
    tarball
}

fn integrity_for(bytes: &[u8]) -> String {
    let digest = Sha512::digest(bytes);
    format!("sha512-{}", BASE64_STANDARD.encode(digest))
}
