//! Tests for M4 policy metadata in inspect JSON output.

use crate::{
    build_authority_context_with_paths, build_inspect_json_report, ArtifactIdentity, CommandIntent,
    Decision, ExtractedPackageMetadata, InspectAuthorityContext, InspectDecision,
    InspectExecutionState, InspectExecutionStateKind, InspectFacts, InspectModel,
    InspectNextAction, M1Evidence, PackageOptionalEvidence, PackageScopeCategory, PackageSpec,
    RegistryEvidence, RegistrySource, Report, ResolvedPackage, SourceContext,
    StaticExtractionEvidence,
};
use std::collections::BTreeMap;
use std::path::Path;

/// Verifies threshold findings are visible in agent-facing JSON metadata.
#[test]
fn threshold_policy_findings_are_visible_in_json() {
    let value = serde_json::to_value(build_inspect_json_report(&threshold_report()))
        .expect("schema should serialize");

    assert_eq!(value["decision"], "ask");
    assert_eq!(value["required_next_action"], "ask_user");
    assert_eq!(
        value["reasons"],
        serde_json::json!([
            "caller_requested_allow",
            "large_package",
            "large_file_count",
            "lifecycle_script_present"
        ])
    );
    assert_eq!(value["policy"]["decision"], "ask");
    assert_eq!(
        value["policy"]["rule_ids"],
        serde_json::json!([
            "caller_recommendation",
            "large_package",
            "large_file_count",
            "lifecycle_script"
        ])
    );
    assert_eq!(value["policy"]["findings"].as_array().unwrap().len(), 3);
    assert_eq!(
        value["decision_receipt"]["evidence_summary"]["reasons"],
        value["reasons"]
    );
}

/// Verifies recent publish evidence produces an agent-visible ask decision.
#[test]
fn recent_publish_policy_ask_is_visible_in_json() {
    let value = serde_json::to_value(build_inspect_json_report(&recent_publish_report()))
        .expect("schema should serialize");

    assert_eq!(value["decision"], "ask");
    assert_eq!(value["required_next_action"], "ask_user");
    assert_eq!(
        value["reasons"],
        serde_json::json!(["caller_requested_allow", "recent_publish"])
    );
    assert_eq!(
        value["policy"]["rule_ids"],
        serde_json::json!(["caller_recommendation", "recent_publish"])
    );
    assert_eq!(value["policy"]["findings"][0]["reason"], "recent_publish");
    assert_eq!(
        value["policy"]["findings"][0]["observed"],
        "published_at=2999-01-01T00:00:00Z; clock_skew_or_future_publish=true"
    );
}

/// Verifies large package evidence alone produces an agent-visible ask decision.
#[test]
fn large_package_policy_ask_is_visible_in_json() {
    let value = serde_json::to_value(build_inspect_json_report(&large_package_report()))
        .expect("schema should serialize");

    assert_eq!(value["decision"], "ask");
    assert_eq!(value["required_next_action"], "ask_user");
    assert_eq!(
        value["reasons"],
        serde_json::json!(["caller_requested_allow", "large_package"])
    );
    assert_eq!(
        value["policy"]["rule_ids"],
        serde_json::json!(["caller_recommendation", "large_package"])
    );
    assert_eq!(value["policy"]["findings"].as_array().unwrap().len(), 1);
    assert_eq!(value["policy"]["findings"][0]["reason"], "large_package");
}

/// Build a verified report with deterministic threshold-triggering evidence.
fn threshold_report() -> Report {
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

    Report {
        package_spec: intent.requested.clone(),
        inspect: InspectModel {
            heuristics: Vec::new(),
            decision: InspectDecision {
                recommendation: Decision::Allow,
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
        recommendation: Decision::Allow,
        status: "m3_inspect",
        note: "fixture",
        m1: verified_m1(artifact),
    }
}

/// Build a verified report with deterministic freshness warning evidence.
fn recent_publish_report() -> Report {
    let mut report = threshold_report();
    let M1Evidence::Verified {
        registry_evidence,
        static_extraction,
        ..
    } = &mut report.m1
    else {
        panic!("fixture should be verified");
    };
    registry_evidence.publish_time = Some("2999-01-01T00:00:00Z".to_string());
    *static_extraction = None;
    report
}

/// Build a verified report with only large-package threshold evidence.
fn large_package_report() -> Report {
    let mut report = threshold_report();
    let M1Evidence::Verified {
        static_extraction, ..
    } = &mut report.m1
    else {
        panic!("fixture should be verified");
    };
    *static_extraction = Some(static_extraction_evidence(
        crate::M4_LARGE_TARBALL_WARNING_BYTES + 1,
        1,
        &[],
    ));
    report
}

/// Build verified M1 evidence that includes threshold-triggering extraction.
fn verified_m1(artifact: ArtifactIdentity) -> M1Evidence {
    M1Evidence::Verified {
        resolved_package: ResolvedPackage {
            name: "create-example".to_string(),
            version: "1.2.3".to_string(),
            registry: registry_source(),
            tarball_url: "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz"
                .to_string(),
            integrity: "sha512-example".to_string(),
        },
        integrity_status: "verified",
        artifact_identity: artifact,
        registry_evidence: RegistryEvidence {
            registry: registry_source(),
            package_scope: PackageScopeCategory::Unscoped,
            name: "create-example".to_string(),
            version: "1.2.3".to_string(),
            publish_time: None,
            maintainers: Vec::new(),
            publisher: None,
            repository: None,
            license: None,
            provenance: BTreeMap::new(),
            dist_integrity: "sha512-example".to_string(),
            tarball_url: "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz"
                .to_string(),
            evidence_boundary: "registry metadata is not proof of tarball package contents",
        },
        static_extraction: Some(static_extraction_with_thresholds()),
    }
}

/// Build static extraction evidence that fires deterministic threshold rules.
fn static_extraction_with_thresholds() -> StaticExtractionEvidence {
    static_extraction_evidence(
        crate::M4_LARGE_TARBALL_WARNING_BYTES + 1,
        crate::M4_LARGE_FILE_COUNT_WARNING + 1,
        &["postinstall"],
    )
}

/// Build static extraction evidence with caller-selected threshold values.
fn static_extraction_evidence(
    artifact_size_bytes: usize,
    file_count: usize,
    lifecycle_script_names: &[&str],
) -> StaticExtractionEvidence {
    let lifecycle_scripts = lifecycle_script_names
        .iter()
        .map(|name| {
            (
                (*name).to_string(),
                "node scripts/postinstall.js".to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    StaticExtractionEvidence {
        metadata: ExtractedPackageMetadata {
            name: Some("create-example".to_string()),
            version: Some("1.2.3".to_string()),
            bins: BTreeMap::new(),
            lifecycle_scripts,
            dependency_declarations: Vec::new(),
            optional_evidence: PackageOptionalEvidence::default(),
            package_json_path: Path::new("package/package.json").to_path_buf(),
        },
        artifact_size_bytes,
        file_count,
        status: "extracted",
    }
}

/// Return the public npm registry source fixture.
fn registry_source() -> RegistrySource {
    RegistrySource {
        url: crate::PUBLIC_NPM_REGISTRY_URL.to_string(),
        scope: None,
    }
}
