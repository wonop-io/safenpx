//! Tests for the shared M3 inspect evidence model.

use crate::{
    build_report_with_resolver, render_report, Cli, InspectExecutionStateKind,
    InspectHeuristicKind, NpmMetadataClient, RegistryHttpResponse, RegistryTransport,
    RegistryTransportError, RootArtifactResolver, TarballDownloader, TarballHttpResponse,
    TarballTransport, TarballTransportError,
};
use base64::prelude::{Engine as _, BASE64_STANDARD};
use clap::Parser;
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
fn fact_only_inspect_evidence_has_no_heuristics() {
    let tarball = package_tarball(
        r#"{"name":"create-example","version":"1.2.3","bin":{"create-example":"bin/create.js"}}"#,
    );
    let report = inspect_report(&tarball);

    assert!(report.inspect.facts.root_package.is_some());
    assert!(report.inspect.heuristics.is_empty());
    assert_eq!(
        report.inspect.execution_state.state,
        InspectExecutionStateKind::StoppedBeforeExecution
    );
    assert!(!report.inspect.execution_state.package_code_executed);
}

#[test]
fn heuristic_evidence_is_report_only_and_not_a_hard_denial() {
    let tarball = package_tarball(
        r#"{
            "name":"create-example",
            "version":"1.2.3",
            "bin":{"create-example":"bin/create.js"},
            "scripts":{"postinstall":"node postinstall.js"},
            "dependencies":{"left-pad":"1.3.0"}
        }"#,
    );
    let report = inspect_report(&tarball);

    assert_eq!(report.inspect.decision.recommendation, crate::Decision::Ask);
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
    assert!(report
        .inspect
        .heuristics
        .iter()
        .any(|signal| signal.kind == InspectHeuristicKind::DependencyDeclarationsPresent));
}

#[test]
fn unsupported_refusal_evidence_is_separate_from_facts() {
    let cli = Cli::parse_from(["safe-npx", "inspect", "create-example@latest"]);
    let report = build_report_with_resolver(&cli, &resolver_with(b"unused".to_vec()));

    assert!(report.inspect.facts.refusal.is_some());
    assert!(report.inspect.facts.root_package.is_none());
    assert_eq!(
        report.inspect.execution_state.state,
        InspectExecutionStateKind::RefusedBeforeExecution
    );
    assert_eq!(
        report.inspect.decision.required_next_action,
        crate::InspectNextAction::Stop
    );
}

#[test]
fn missing_optional_facts_remain_absent() {
    let tarball = package_tarball(r#"{"name":"create-example","version":"1.2.3"}"#);
    let report = inspect_report(&tarball);
    let root_package = report
        .inspect
        .facts
        .root_package
        .expect("inspect should extract root package facts");

    assert_eq!(root_package.metadata.optional_evidence.repository, None);
    assert_eq!(root_package.metadata.optional_evidence.license, None);
    assert!(root_package
        .metadata
        .optional_evidence
        .maintainers
        .is_empty());
}

#[test]
fn human_and_json_renderers_consume_shared_model() {
    let tarball = package_tarball(
        r#"{
            "name":"create-example",
            "version":"1.2.3",
            "bin":{"create-example":"bin/create.js"},
            "scripts":{"postinstall":"node postinstall.js"},
            "dependencies":{"left-pad":"1.3.0"}
        }"#,
    );
    let resolver = resolver_with(tarball);
    let human_cli = Cli::parse_from(["safe-npx", "inspect", "create-example@1.2.3"]);
    let json_cli = Cli::parse_from(["safe-npx", "--json", "inspect", "create-example@1.2.3"]);
    let human_report = build_report_with_resolver(&human_cli, &resolver);
    let human = render_report(&human_cli, &human_report).expect("human report should render");
    let json_report = build_report_with_resolver(
        &json_cli,
        &resolver_with(package_tarball(
            r#"{
            "name":"create-example",
            "version":"1.2.3",
            "bin":{"create-example":"bin/create.js"},
            "scripts":{"postinstall":"node postinstall.js"},
            "dependencies":{"left-pad":"1.3.0"}
        }"#,
        )),
    );
    let json = render_report(&json_cli, &json_report).expect("json report should render");

    assert!(human.contains("postinstall -> node postinstall.js"));
    assert!(human.contains("left-pad (Runtime) 1.3.0 [declaration_only]"));
    assert!(human.contains("- lifecycle_scripts_present [report_only]"));
    assert!(human.contains("- dependency_declarations_present [report_only]"));
    assert!(human.contains("Decision reasons: m3_heuristics_report_only"));
    assert!(human.contains("Required next action: ask_user"));
    assert!(human.contains("Authority: command=create-example@1.2.3"));
    assert!(human.contains("Execution: stopped_before_execution; package code executed: false"));
    assert!(json.contains("\"inspect\""));
    assert!(json.contains("\"heuristics\""));
    assert!(json.contains("\"report_only\": true"));
}

#[test]
fn human_renderer_preserves_failed_refusal_state_from_shared_model() {
    let tarball = package_tarball(r#"{"name":"create-example","version":"1.2.3"}"#);
    let resolver = RootArtifactResolver::new(
        NpmMetadataClient::public(StubRegistryTransport::ok(metadata_body(&integrity_for(
            b"different bytes",
        )))),
        TarballDownloader::new(StubTarballTransport::ok(tarball)),
    );
    let cli = Cli::parse_from(["safe-npx", "inspect", "create-example@1.2.3"]);
    let report = build_report_with_resolver(&cli, &resolver);
    let human = render_report(&cli, &report).expect("human report should render");

    assert_eq!(
        report.inspect.execution_state.state,
        InspectExecutionStateKind::FailedBeforeExecution
    );
    assert!(human.contains("M1 evidence: failed"));
    assert!(human.contains("Reason: integrity_mismatch"));
    assert!(!human.contains("M1 evidence: no_download"));
}

fn inspect_report(tarball: &[u8]) -> crate::Report {
    let cli = Cli::parse_from(["safe-npx", "inspect", "create-example@1.2.3"]);
    build_report_with_resolver(&cli, &resolver_with(tarball.to_vec()))
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
