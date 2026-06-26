//! Tests for report and authority-context redaction boundaries.

use crate::{
    build_authority_context_with_paths, build_report, redact_report_value, render_report,
    ArtifactIdentity, Cli, CommandIntent, Decision, InspectAuthorityContext, InspectDecision,
    InspectExecutionState, InspectExecutionStateKind, InspectFacts, InspectModel,
    InspectNextAction, M1Evidence, M1Reason, PackageScopeCategory, PackageSpec, RegistryEvidence,
    RegistryPerson, RegistrySource, Report, ResolvedPackage, SourceContext, UnsupportedSpec,
    UnsupportedSpecCategory,
};
use std::collections::BTreeMap;
use std::path::Path;

const FIXTURES: &str = include_str!("../fixtures/authority-redaction-fixture-manifest.txt");

/// Proves the golden redaction fixture manifest is consumed by tests.
#[test]
fn authority_redaction_fixture_manifest_is_enforced() {
    let mut fixture_count = 0;
    for line in FIXTURES.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        fixture_count += 1;
        let columns = line.split('|').collect::<Vec<_>>();
        assert_eq!(columns.len(), 5, "invalid fixture row: {line}");
        let actual = render_fixture(columns[1], columns[2]);
        assert!(
            !actual.contains(columns[3]),
            "fixture {} leaked {} in {actual}",
            columns[0],
            columns[3]
        );
        assert!(
            actual.contains(columns[4]),
            "fixture {} missed {} in {actual}",
            columns[0],
            columns[4]
        );
    }
    assert!(fixture_count >= 8);
}

/// Proves unsupported local specs are redacted in human and JSON reports.
#[test]
fn unsupported_specs_are_redacted_in_human_and_json_reports() {
    let human_cli = Cli::try_parse_from(["safe-npx", "--dry-run", "/Users/alice/private/pkg"])
        .expect("local path should parse as unsupported input");
    let human = render_report(&human_cli, &build_report(&human_cli)).expect("render should work");
    assert!(!human.contains("/Users/alice"));
    assert!(human.contains("<home>"));

    let json_cli = Cli::try_parse_from([
        "safe-npx",
        "--json",
        "--dry-run",
        "/Users/alice/private/pkg",
    ])
    .expect("local path should parse as unsupported input");
    let json = render_report(&json_cli, &build_report(&json_cli)).expect("render should work");
    assert!(!json.contains("/Users/alice"));
    assert!(json.contains("<home>"));
}

/// Proves verified reports redact raw registry and tarball values in both outputs.
#[test]
fn verified_reports_redact_registry_and_tarball_values() {
    let report = verified_report_with_secret_urls();
    let json = serde_json::to_string_pretty(&report).expect("report should serialize");
    assert_report_is_redacted(&json);

    let cli = Cli::try_parse_from(["safe-npx", "--dry-run", "create-example@1.2.3"])
        .expect("exact spec should parse");
    let human = render_report(&cli, &report).expect("human render should work");
    assert_report_is_redacted(&human);
}

/// Render one fixture input through the same redaction boundary as reports.
fn render_fixture(kind: &str, input: &str) -> String {
    match kind {
        "report_value" => redact_report_value(input),
        "cwd" => {
            let context = build_authority_context_with_paths(
                "create-example@1.2.3",
                &SourceContext::Unknown,
                None,
                Some("unscoped".to_string()),
                Some(Path::new(input)),
                Some(Path::new("/Users/alice")),
            );
            format!("{} {}", context.cwd.category, context.cwd.display)
        }
        "source_context" => {
            let source_context = match input {
                "ci" => SourceContext::Ci,
                "agent_skill" => SourceContext::AgentSkill,
                other => panic!("unknown source context fixture: {other}"),
            };
            let context = build_authority_context_with_paths(
                "create-example@1.2.3",
                &source_context,
                None,
                Some("unscoped".to_string()),
                Some(Path::new("/workspace/repo")),
                Some(Path::new("/Users/alice")),
            );
            let runner = match context.runner_context {
                crate::AuthorityRunnerContext::Ci => "ci",
                crate::AuthorityRunnerContext::Agent => "agent",
                crate::AuthorityRunnerContext::LocalTerminal => "local_terminal",
                crate::AuthorityRunnerContext::Unknown => "unknown",
            };
            let actor = match context.actor_context {
                crate::AuthorityActorContext::Automation => "automation",
                crate::AuthorityActorContext::CodingAgent => "coding_agent",
                crate::AuthorityActorContext::ManualUser => "manual_user",
                crate::AuthorityActorContext::Unknown => "unknown",
            };
            format!("{runner} {actor}")
        }
        "registry" => {
            let registry = RegistrySource {
                url: input.to_string(),
                scope: Some("scope".to_string()),
            };
            let context = build_authority_context_with_paths(
                "@scope/create-example@1.2.3",
                &SourceContext::Unknown,
                Some(&registry),
                Some("scoped".to_string()),
                Some(Path::new("/workspace/repo")),
                Some(Path::new("/Users/alice")),
            );
            context
                .registry
                .expect("registry fixture should include registry")
                .display_url
        }
        other => panic!("unknown fixture kind: {other}"),
    }
}

/// Build a report that contains secrets in every sensitive report surface.
fn verified_report_with_secret_urls() -> Report {
    let registry = RegistrySource {
        url: "https://secret-token@registry.example.test/npm/?_authToken=secret-token".to_string(),
        scope: Some("scope".to_string()),
    };
    let tarball_url = "https://registry.example.test/pkg.tgz?token=secret-token&ok=1".to_string();
    let resolved_package = ResolvedPackage {
        name: "@scope/create-example".to_string(),
        version: "1.2.3".to_string(),
        registry: registry.clone(),
        tarball_url: tarball_url.clone(),
        integrity: "sha512-example".to_string(),
    };
    let registry_evidence = RegistryEvidence {
        registry: registry.clone(),
        package_scope: PackageScopeCategory::Scoped,
        name: "@scope/create-example".to_string(),
        version: "1.2.3".to_string(),
        publish_time: Some("2026-06-26T00:00:00Z".to_string()),
        maintainers: vec![RegistryPerson {
            name: Some("maintainer".to_string()),
            email: None,
        }],
        publisher: None,
        repository: Some("https://secret-token@github.example.test/org/repo".to_string()),
        license: Some("Apache-2.0".to_string()),
        provenance: BTreeMap::new(),
        dist_integrity: "sha512-example".to_string(),
        tarball_url,
        evidence_boundary: "registry metadata is not proof of tarball package contents",
    };
    let artifact_identity = ArtifactIdentity {
        name: "@scope/create-example".to_string(),
        version: "1.2.3".to_string(),
        integrity: "sha512-example".to_string(),
        digest_algorithm: "sha512".to_string(),
        digest: "digest-example".to_string(),
    };
    let intent = CommandIntent::supported(
        PackageSpec::exact(
            "@scope/create-example@1.2.3",
            "@scope/create-example",
            "1.2.3",
            Some("scope".to_string()),
        ),
        vec![
            "--token".to_string(),
            "secret-token".to_string(),
            "--cwd".to_string(),
            "/Users/alice/private/project".to_string(),
            "--password=secret-token".to_string(),
            "--config=//registry.example.test/:_authToken=secret-token".to_string(),
        ],
    );
    let authority = build_authority_context_with_paths(
        &intent.requested,
        &SourceContext::AgentSkill,
        Some(&registry),
        Some("scoped".to_string()),
        Some(Path::new("/workspace/customer/private-repo")),
        Some(Path::new("/Users/alice")),
    );
    let inspect = InspectModel {
        facts: InspectFacts {
            command: intent.clone(),
            resolved_package: Some(resolved_package.clone()),
            registry: Some(registry_evidence.clone()),
            artifact: Some(artifact_identity.clone()),
            root_package: None,
            refusal: None,
        },
        heuristics: Vec::new(),
        decision: InspectDecision {
            recommendation: Decision::Ask,
            reasons: vec!["test".to_string()],
            required_next_action: InspectNextAction::AskUser,
        },
        authority_context: InspectAuthorityContext {
            redacted: authority,
        },
        execution_state: InspectExecutionState {
            state: InspectExecutionStateKind::StoppedBeforeExecution,
            package_code_executed: false,
        },
    };

    Report {
        package_spec: "https://secret-token@registry.example.test/pkg.tgz".to_string(),
        intent,
        recommendation: Decision::Ask,
        status: "m3_inspect",
        note: "test report",
        inspect,
        m1: M1Evidence::Verified {
            resolved_package,
            integrity_status: "verified",
            artifact_identity,
            registry_evidence,
            static_extraction: None,
        },
    }
}

/// Assert report output hides credentials, tokens, and local machine paths.
fn assert_report_is_redacted(output: &str) {
    assert!(!output.contains("secret-token"));
    assert!(!output.contains("/workspace/customer/private-repo"));
    assert!(!output.contains("/Users/alice"));
    assert!(output.contains("<redacted>") || output.contains("_authToken=<redacted>"));
}

/// Proves unsupported tarball specs are redacted when serialized directly.
#[test]
fn unsupported_tarball_intent_serialization_is_redacted() {
    let intent = CommandIntent::unsupported(
        "https://secret-token@registry.example.test/pkg.tgz",
        UnsupportedSpec {
            reason: M1Reason::UnsupportedSpec,
            category: UnsupportedSpecCategory::TarballUrl,
            downloaded: false,
        },
    );
    let json = serde_json::to_string(&intent).expect("intent should serialize");
    assert!(!json.contains("secret-token"));
    assert!(json.contains("<redacted>"));
}
