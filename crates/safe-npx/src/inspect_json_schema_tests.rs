//! Tests for the M3 inspect JSON schema contract.

use crate::{
    build_authority_context_with_paths, build_inspect_json_report,
    build_m2_execution_refusal_json_report, build_m2_execution_refusal_report, render_report, run,
    ArtifactIdentity, Cli, ClosureCommandIdentity, CommandIntent, Decision,
    InspectAuthorityContext, InspectDecision, InspectExecutionState, InspectExecutionStateKind,
    InspectFacts, InspectJsonDecision, InspectJsonNextAction, InspectModel, InspectNextAction,
    InspectRefusalFact, InspectRefusalState, M1Evidence, M1Reason, M2Reason, PackageSpec,
    RegistrySource, Report, SourceContext, UnsupportedSpec, UnsupportedSpecCategory,
};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

const ASK_GOLDEN: &str = include_str!("../fixtures/inspect-json-schema-v0-ask.json");
const COMPATIBILITY_GOLDEN: &str =
    include_str!("../fixtures/inspect-json-schema-v0-compatibility.json");
const UNSUPPORTED_GOLDEN: &str =
    include_str!("../fixtures/inspect-json-schema-v0-unsupported.json");
const FAILURE_GOLDEN: &str = include_str!("../fixtures/inspect-json-schema-v0-failure.json");

/// Verifies required top-level fields and reserved nulls for ask-style output.
#[test]
fn inspect_json_schema_has_required_top_level_shape() {
    let report = verified_ask_report();
    let value =
        serde_json::to_value(build_inspect_json_report(&report)).expect("schema should serialize");

    for field in [
        "schema_version",
        "artifact",
        "command_intent",
        "source_context",
        "authority_context",
        "facts",
        "heuristics",
        "external_evidence",
        "attestations",
        "release_diff",
        "decision_receipt",
        "decision",
        "reasons",
        "required_next_action",
        "execution",
        "exit_code",
    ] {
        assert!(value.get(field).is_some(), "missing field {field}");
    }

    assert_eq!(value["schema_version"], "0.1");
    assert_eq!(value["decision"], "ask");
    assert_eq!(value["required_next_action"], "ask_user");
    assert_reserved_fields_are_null(&value);
    assert!(value["decision_receipt"].is_object());
    assert_eq!(value["execution"], Value::Null);
    assert_eq!(value["exit_code"], 0);
}

/// Verifies unsupported input uses the M3 unsupported decision vocabulary.
#[test]
fn unsupported_json_uses_m3_decision_and_retry_action() {
    let cli = Cli::parse_from(["safe-npx", "--json", "create-example@next"]);
    let value: Value =
        serde_json::from_str(&run(&cli).expect("json should render")).expect("json should parse");

    assert_eq!(value["schema_version"], "0.1");
    assert_eq!(value["decision"], "unsupported");
    assert_eq!(value["required_next_action"], "retry_narrower_command");
    assert_reserved_fields_are_null(&value);
    assert_eq!(value["execution"], Value::Null);
    assert!(value["facts"]["refusal"].is_object());
}

/// Verifies inspection failures use the M3 inspection-error vocabulary.
#[test]
fn failed_inspection_json_uses_inspection_error_decision() {
    let report = failed_report();
    let value =
        serde_json::to_value(build_inspect_json_report(&report)).expect("schema should serialize");

    assert_eq!(value["decision"], "inspection_error");
    assert_eq!(value["required_next_action"], "inspect_only");
    assert_reserved_fields_are_null(&value);
    assert_eq!(value["execution"], Value::Null);
    assert_eq!(value["exit_code"], 3);
}

/// Verifies execution-refused JSON also uses the M3 schema envelope.
#[test]
fn execution_refusal_json_uses_m3_schema_envelope() {
    let report = build_m2_execution_refusal_report(
        ClosureCommandIdentity {
            requested: "create-example@1.2.3".to_string(),
            forwarded_args: vec!["--template".to_string(), "react".to_string()],
        },
        vec![M2Reason::UnsupportedClosure],
    );
    let value = build_m2_execution_refusal_json_report(&report);

    assert_eq!(value["schema_version"], "0.1");
    assert_eq!(value["decision"], "execution_refused");
    assert_eq!(value["required_next_action"], "inspect_only");
    assert_eq!(value["execution"], Value::Null);
    assert!(value.get("authority_context").is_some());
    assert_reserved_fields_are_null(&value);
    assert_eq!(value.get("decision_receipt"), Some(&Value::Null));
}

/// Verifies checked-in base fixtures keep future hosted evidence fields null.
#[test]
fn base_schema_fixtures_keep_reserved_fields_null() {
    for (name, fixture) in [
        ("inspect-json-schema-v0-ask.json", ASK_GOLDEN),
        (
            "inspect-json-schema-v0-unsupported.json",
            UNSUPPORTED_GOLDEN,
        ),
        ("inspect-json-schema-v0-failure.json", FAILURE_GOLDEN),
    ] {
        let value: Value = serde_json::from_str(fixture).expect("fixture should parse");
        assert_reserved_fields_are_null(&value);
        assert_eq!(value["execution"], Value::Null, "{name}");
    }
}

/// Verifies enum spelling stays aligned with `docs/milestones.md`.
#[test]
fn inspect_json_enum_vocabulary_is_stable() {
    let decisions = serde_json::to_value([
        InspectJsonDecision::Allow,
        InspectJsonDecision::Ask,
        InspectJsonDecision::Deny,
        InspectJsonDecision::Unsupported,
        InspectJsonDecision::InspectionError,
        InspectJsonDecision::ExecutionRefused,
    ])
    .expect("decisions should serialize");
    let actions = serde_json::to_value([
        InspectJsonNextAction::None,
        InspectJsonNextAction::AskUser,
        InspectJsonNextAction::RetryNarrowerCommand,
        InspectJsonNextAction::InspectOnly,
        InspectJsonNextAction::ExplicitOverride,
        InspectJsonNextAction::Unsupported,
    ])
    .expect("actions should serialize");

    assert_eq!(
        decisions,
        serde_json::json!([
            "allow",
            "ask",
            "deny",
            "unsupported",
            "inspection_error",
            "execution_refused"
        ])
    );
    assert_eq!(
        actions,
        serde_json::json!([
            "none",
            "ask_user",
            "retry_narrower_command",
            "inspect_only",
            "explicit_override",
            "unsupported"
        ])
    );
}

/// Verifies enum vocabulary and semantic mappings stay reviewable.
#[test]
fn inspect_json_schema_compatibility_manifest_is_stable() {
    let manifest = serde_json::to_string_pretty(&compatibility_manifest())
        .expect("compatibility manifest should serialize");
    let manifest = format!("{manifest}\n");
    let cases = [(
        "inspect-json-schema-v0-compatibility.json",
        manifest.as_str(),
    )];

    if maybe_update_schema_goldens(&cases) {
        return;
    }
    assert_schema_golden(COMPATIBILITY_GOLDEN, &manifest);
}

/// Verifies compatibility-critical schema fields stay aligned with golden fixtures.
#[test]
fn inspect_json_schema_golden_fixtures_are_stable() {
    let json_cli = Cli::parse_from(["safe-npx", "--json", "create-example@1.2.3"]);
    let ask = render_schema_golden(&json_cli, &verified_ask_report());
    let unsupported = render_schema_golden(&json_cli, &unsupported_report());
    let failure = render_schema_golden(&json_cli, &failed_report());

    let cases = [
        ("inspect-json-schema-v0-ask.json", ask.as_str()),
        (
            "inspect-json-schema-v0-unsupported.json",
            unsupported.as_str(),
        ),
        ("inspect-json-schema-v0-failure.json", failure.as_str()),
    ];
    maybe_print_schema_goldens(&cases);
    if maybe_update_schema_goldens(&cases) {
        return;
    }

    assert_schema_golden(ASK_GOLDEN, &ask);
    assert_schema_golden(UNSUPPORTED_GOLDEN, &unsupported);
    assert_schema_golden(FAILURE_GOLDEN, &failure);
}

/// Builds a stable compatibility manifest for enum and semantic mapping review.
fn compatibility_manifest() -> Value {
    let m2_refusal = build_m2_execution_refusal_report(
        ClosureCommandIdentity {
            requested: "create-example@1.2.3".to_string(),
            forwarded_args: Vec::new(),
        },
        vec![M2Reason::LifecycleScriptPresent],
    );

    serde_json::json!({
        "schema_version": "0.1",
        "decisions": [
            "allow",
            "ask",
            "deny",
            "unsupported",
            "inspection_error",
            "execution_refused"
        ],
        "required_next_actions": [
            "none",
            "ask_user",
            "retry_narrower_command",
            "inspect_only",
            "explicit_override",
            "unsupported"
        ],
        "semantic_cases": {
            "verified_ask": semantic_case(build_inspect_json_report(&verified_ask_report())),
            "unsupported_input": semantic_case(build_inspect_json_report(&unsupported_report())),
            "inspection_failure": semantic_case(build_inspect_json_report(&failed_report())),
            "execution_refusal": semantic_case(build_m2_execution_refusal_json_report(&m2_refusal)),
        },
        "compatibility_rules": {
            "additive_fields_allowed_within_0x": true,
            "enum_additions_require_schema_bump": true,
            "enum_semantic_changes_require_migration_note": true
        }
    })
}

/// Extracts the compatibility-critical fields from JSON schema output.
fn semantic_case(value: impl Serialize) -> Value {
    let value = serde_json::to_value(value).expect("schema output should serialize");
    serde_json::json!({
        "decision": value["decision"],
        "required_next_action": value["required_next_action"],
        "exit_code": value["exit_code"],
    })
}

/// Assert the hosted-evidence fields are reserved and unpopulated in V0.
fn assert_reserved_fields_are_null(value: &Value) {
    for field in ["external_evidence", "attestations", "release_diff"] {
        assert_eq!(
            value.get(field),
            Some(&Value::Null),
            "{field} must be present and null in V0"
        );
    }
}

/// Renders a schema golden with a single trailing newline.
fn render_schema_golden(cli: &Cli, report: &Report) -> String {
    format!(
        "{}\n",
        render_report(cli, report).expect("schema should render")
    )
}

/// Prints regenerated fixture contents when explicitly requested.
fn maybe_print_schema_goldens(cases: &[(&str, &str)]) {
    if std::env::var_os("SAFE_NPX_PRINT_SCHEMA_GOLDENS").is_none() {
        return;
    }

    for (name, value) in cases {
        eprintln!("--- {name} ---\n{value}");
    }
}

/// Updates base schema fixture files when explicitly requested.
fn maybe_update_schema_goldens(cases: &[(&str, &str)]) -> bool {
    if std::env::var_os("SAFE_NPX_UPDATE_SCHEMA_GOLDENS").is_none() {
        return false;
    }

    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    for (name, value) in cases {
        std::fs::write(fixture_root.join(name), value).expect("fixture should update");
    }
    true
}

/// Compares full pretty-printed schema JSON against a checked-in fixture.
fn assert_schema_golden(expected: &str, actual: &str) {
    assert_eq!(actual, expected);
}

/// Builds a successful inspect report that stops before execution and asks.
fn verified_ask_report() -> Report {
    let intent = CommandIntent::supported(
        PackageSpec::exact("create-example@1.2.3", "create-example", "1.2.3", None),
        Vec::new(),
    );
    let artifact = ArtifactIdentity {
        name: "create-example".to_string(),
        version: "1.2.3".to_string(),
        integrity: "sha512-example".to_string(),
        digest_algorithm: "sha512".to_string(),
        digest: "digest-example".to_string(),
    };
    let facts = InspectFacts {
        command: intent.clone(),
        resolved_package: None,
        registry: None,
        artifact: Some(artifact.clone()),
        root_package: None,
        refusal: None,
    };

    report_from_parts(
        intent,
        Decision::Ask,
        facts,
        M1Evidence::Verified {
            resolved_package: crate::ResolvedPackage {
                name: "create-example".to_string(),
                version: "1.2.3".to_string(),
                registry: RegistrySource {
                    url: crate::PUBLIC_NPM_REGISTRY_URL.to_string(),
                    scope: None,
                },
                tarball_url: "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz"
                    .to_string(),
                integrity: "sha512-example".to_string(),
            },
            integrity_status: "verified",
            artifact_identity: artifact,
            registry_evidence: crate::RegistryEvidence {
                registry: RegistrySource {
                    url: crate::PUBLIC_NPM_REGISTRY_URL.to_string(),
                    scope: None,
                },
                package_scope: crate::PackageScopeCategory::Unscoped,
                name: "create-example".to_string(),
                version: "1.2.3".to_string(),
                publish_time: None,
                maintainers: Vec::new(),
                publisher: None,
                repository: None,
                license: None,
                provenance: std::collections::BTreeMap::new(),
                dist_integrity: "sha512-example".to_string(),
                tarball_url: "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz"
                    .to_string(),
                evidence_boundary: "registry metadata is not proof of tarball package contents",
            },
            static_extraction: None,
        },
    )
}

/// Builds an unsupported inspect report without downloading package bytes.
fn unsupported_report() -> Report {
    let intent = CommandIntent::unsupported(
        "create-example@next",
        UnsupportedSpec {
            reason: M1Reason::UnsupportedSpec,
            category: UnsupportedSpecCategory::VersionRange,
            downloaded: false,
        },
    );
    let facts = InspectFacts {
        command: intent.clone(),
        resolved_package: None,
        registry: None,
        artifact: None,
        root_package: None,
        refusal: Some(InspectRefusalFact {
            state: InspectRefusalState::NoDownload,
            reason: M1Reason::UnsupportedSpec,
            downloaded: false,
            detail: Some("unsupported spec".to_string()),
        }),
    };

    report_from_parts(
        intent,
        Decision::Ask,
        facts,
        M1Evidence::NoDownload {
            reason: M1Reason::UnsupportedSpec,
            downloaded: false,
        },
    )
}

/// Builds a failed inspect report that maps to the inspection-error vocabulary.
fn failed_report() -> Report {
    let intent = CommandIntent::supported(
        PackageSpec::exact("create-example@1.2.3", "create-example", "1.2.3", None),
        Vec::new(),
    );
    let facts = InspectFacts {
        command: intent.clone(),
        resolved_package: None,
        registry: None,
        artifact: None,
        root_package: None,
        refusal: Some(InspectRefusalFact {
            state: InspectRefusalState::Failed,
            reason: M1Reason::RegistryError,
            downloaded: false,
            detail: Some("registry unavailable".to_string()),
        }),
    };

    report_from_parts(
        intent,
        Decision::Ask,
        facts,
        M1Evidence::Failed {
            reason: M1Reason::RegistryError,
            downloaded: false,
            detail: "registry unavailable".to_string(),
        },
    )
}

/// Builds a report from shared inspect model parts for schema contract tests.
fn report_from_parts(
    intent: CommandIntent,
    recommendation: Decision,
    facts: InspectFacts,
    m1: M1Evidence,
) -> Report {
    Report {
        package_spec: intent.requested.clone(),
        inspect: InspectModel {
            heuristics: Vec::new(),
            decision: InspectDecision {
                recommendation: recommendation.clone(),
                reasons: vec!["fixture".to_string()],
                required_next_action: InspectNextAction::AskUser,
            },
            authority_context: InspectAuthorityContext {
                redacted: build_authority_context_with_paths(
                    &intent.requested,
                    &SourceContext::Unknown,
                    None,
                    Some("unscoped".to_string()),
                    Some(Path::new("/workspace/safenpx")),
                    Some(Path::new("/home/example")),
                ),
            },
            execution_state: InspectExecutionState {
                state: InspectExecutionStateKind::StoppedBeforeExecution,
                package_code_executed: false,
            },
            facts,
        },
        intent,
        recommendation,
        status: "m3_inspect",
        note: "fixture",
        m1,
    }
}
