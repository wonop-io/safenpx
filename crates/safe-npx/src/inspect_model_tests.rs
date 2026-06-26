//! Tests for the shared M3 inspect evidence model.

use crate::{
    build_authority_context_with_paths, build_report_with_resolver, render_report,
    AuthorityActorContext, AuthorityRegistryCategory, AuthorityRunnerContext, Cli,
    InspectExecutionStateKind, InspectHeuristicKind, NpmMetadataClient, RegistryHttpResponse,
    RegistrySource, RegistryTransport, RegistryTransportError, RootArtifactResolver, SourceContext,
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
    assert!(human.contains("Authority boundary: authority context describes ambient process authority; it is not sandboxing"));
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

#[test]
fn source_context_defaults_to_unknown_without_inference() {
    let cli = Cli::parse_from(["safe-npx", "inspect", "create-example@latest"]);
    let report = build_report_with_resolver(&cli, &resolver_with(b"unused".to_vec()));
    let human = render_report(&cli, &report).expect("human report should render");

    assert_eq!(
        report.inspect.authority_context.redacted.source_context,
        SourceContext::Unknown
    );
    assert!(human.contains("source_context=unknown"));
}

#[test]
fn source_context_categories_render_in_human_and_json_reports() {
    let cases = [
        ("manual_terminal", SourceContext::ManualTerminal),
        ("docs_snippet", SourceContext::DocsSnippet),
        ("agent_skill", SourceContext::AgentSkill),
        ("ci", SourceContext::Ci),
        ("unknown", SourceContext::Unknown),
    ];

    for (value, expected) in cases {
        let cli = Cli::parse_from([
            "safe-npx",
            "--source-context",
            value,
            "inspect",
            "create-example@latest",
        ]);
        let report = build_report_with_resolver(&cli, &resolver_with(b"unused".to_vec()));
        let human = render_report(&cli, &report).expect("human report should render");

        assert_eq!(
            report.inspect.authority_context.redacted.source_context,
            expected
        );
        assert!(human.contains(&format!("source_context={value}")));

        let json_cli = Cli::parse_from([
            "safe-npx",
            "--json",
            "--source-context",
            value,
            "inspect",
            "create-example@latest",
        ]);
        let json_report = build_report_with_resolver(&json_cli, &resolver_with(b"unused".to_vec()));
        let json = render_report(&json_cli, &json_report).expect("json report should render");

        assert!(json.contains(&format!("\"source_context\": \"{value}\"")));
    }
}

#[test]
fn source_context_after_inspect_action_is_still_caller_declared() {
    let cli = Cli::parse_from([
        "safe-npx",
        "inspect",
        "--source-context",
        "ci",
        "create-example@latest",
    ]);
    let report = build_report_with_resolver(&cli, &resolver_with(b"unused".to_vec()));
    let human = render_report(&cli, &report).expect("human report should render");

    assert_eq!(
        report.inspect.authority_context.redacted.source_context,
        SourceContext::Ci
    );
    assert_eq!(cli.raw_package_spec(), "create-example@latest");
    assert!(human.contains("source_context=ci"));
}

#[test]
fn source_context_after_inspect_action_supports_equals_syntax() {
    let cli = Cli::parse_from([
        "safe-npx",
        "inspect",
        "--source-context=agent_skill",
        "create-example@latest",
    ]);
    let report = build_report_with_resolver(&cli, &resolver_with(b"unused".to_vec()));

    assert_eq!(
        report.inspect.authority_context.redacted.source_context,
        SourceContext::AgentSkill
    );
    assert_eq!(cli.raw_package_spec(), "create-example@latest");
}

#[test]
fn invalid_source_context_fails_closed_at_cli_parse_time() {
    let error = Cli::try_parse_from([
        "safe-npx",
        "--source-context",
        "guessed_terminal",
        "inspect",
        "create-example@latest",
    ])
    .expect_err("invalid source context should not parse");

    assert_eq!(error.kind(), clap::error::ErrorKind::InvalidValue);
}

#[test]
fn invalid_source_context_after_inspect_fails_closed_at_cli_parse_time() {
    let error = Cli::try_parse_from([
        "safe-npx",
        "inspect",
        "--source-context",
        "guessed_terminal",
        "create-example@latest",
    ])
    .expect_err("invalid source context should not parse after inspect");

    assert_eq!(error.kind(), clap::error::ErrorKind::InvalidValue);
}

#[test]
fn authority_context_redacts_registry_tokens_and_separates_identity_fields() {
    let registry = RegistrySource {
        url: "https://secret-token@registry.example.test/npm/".to_string(),
        scope: Some("@scope".to_string()),
    };
    let authority = build_authority_context_with_paths(
        "@scope/create-example@1.2.3",
        &SourceContext::ManualTerminal,
        Some(&registry),
        Some("scoped".to_string()),
        Some(std::path::Path::new("/workspace/project")),
        Some(std::path::Path::new("/Users/alice")),
    );
    let registry = authority.registry.expect("registry should render");

    assert_eq!(registry.category, AuthorityRegistryCategory::ScopedRegistry);
    assert_eq!(
        registry.display_url,
        "https://<redacted>@registry.example.test/npm/"
    );
    assert!(!registry.display_url.contains("secret-token"));
    assert_eq!(
        authority.identity.status,
        "reserved_for_canonical_receipt_identity"
    );
}

#[test]
fn authority_context_redacts_home_paths_and_classifies_temp_directories() {
    let home = std::path::Path::new("/Users/alice");
    let home_authority = build_authority_context_with_paths(
        "/Users/alice/project/create-example@1.2.3",
        &SourceContext::DocsSnippet,
        None,
        None,
        Some(std::path::Path::new("/Users/alice/project")),
        Some(home),
    );
    let temp_authority = build_authority_context_with_paths(
        "create-example@1.2.3",
        &SourceContext::Ci,
        None,
        None,
        Some(&std::env::temp_dir().join("safe-npx-test")),
        Some(home),
    );

    assert_eq!(home_authority.cwd.category, "home_subtree");
    assert_eq!(home_authority.cwd.display, "<home>/project");
    assert_eq!(
        home_authority.command_intent.display,
        "<home>/project/create-example@1.2.3"
    );
    assert!(!home_authority.cwd.display.contains("/Users/alice"));
    assert_eq!(temp_authority.cwd.category, "temp_directory");
}

#[test]
fn authority_context_covers_public_npm_ci_and_agent_categories() {
    let registry = RegistrySource {
        url: crate::PUBLIC_NPM_REGISTRY_URL.to_string(),
        scope: None,
    };
    let ci_authority = build_authority_context_with_paths(
        "create-example@1.2.3",
        &SourceContext::Ci,
        Some(&registry),
        Some("unscoped".to_string()),
        Some(std::path::Path::new("/workspace/project")),
        None,
    );
    let agent_authority = build_authority_context_with_paths(
        "create-example@1.2.3",
        &SourceContext::AgentSkill,
        None,
        None,
        Some(std::path::Path::new("/workspace/project")),
        None,
    );

    assert_eq!(ci_authority.runner_context, AuthorityRunnerContext::Ci);
    assert_eq!(
        ci_authority.actor_context,
        AuthorityActorContext::Automation
    );
    assert_eq!(
        ci_authority.registry.expect("registry").category,
        AuthorityRegistryCategory::PublicNpm
    );
    assert_eq!(
        agent_authority.runner_context,
        AuthorityRunnerContext::Agent
    );
    assert_eq!(
        agent_authority.actor_context,
        AuthorityActorContext::CodingAgent
    );
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
