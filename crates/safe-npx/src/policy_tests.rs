//! Policy model regression tests for M4 decision semantics.

use super::*;
use crate::policy_time::parse_rfc3339_utc_seconds;
use crate::{
    ArtifactIdentity, ExtractedPackageMetadata, PackageOptionalEvidence, PackageScopeCategory,
    RegistryEvidence, RegistrySource, ResolvedPackage,
};
use std::collections::BTreeMap;
use std::path::Path;

#[test]
/// Verified evidence preserves the caller-selected ask recommendation.
fn verified_evidence_uses_caller_recommendation_policy() {
    let policy = evaluate_m1_policy(&Decision::Ask, &verified_m1());

    assert_eq!(policy.policy_version, M4_POLICY_VERSION);
    assert_eq!(policy.decision, PolicyDecision::Ask);
    assert_eq!(policy.reasons, vec![PolicyReason::CallerRequestedAsk]);
    assert_eq!(policy.required_next_action, PolicyNextAction::AskUser);
    assert_eq!(policy.rule_ids, vec![PolicyRuleId::CallerRecommendation]);
    assert!(policy.findings.is_empty());
}

#[test]
/// Recent package publication escalates allow to ask with evidence.
fn recent_publish_threshold_escalates_allow_to_ask() {
    let now = parse_rfc3339_utc_seconds("2026-07-01T12:00:00Z").unwrap();
    let policy = evaluate_m1_policy_at(
        &Decision::Allow,
        &verified_m1_with_publish_time("2026-07-01T01:00:00.000Z"),
        now,
    );

    assert_eq!(policy.decision, PolicyDecision::Ask);
    assert_eq!(policy.required_next_action, PolicyNextAction::AskUser);
    assert!(policy.reasons.contains(&PolicyReason::RecentPublish));
    assert!(policy.rule_ids.contains(&PolicyRuleId::RecentPublish));
    assert_eq!(
        policy.findings[0].observed,
        "published_at=2026-07-01T01:00:00.000Z; age_hours=11"
    );
    assert_eq!(policy.findings[0].threshold, "<24h");
}

#[test]
/// Large verified tarballs produce a structured policy warning.
fn large_tarball_threshold_produces_structured_warning() {
    let policy = evaluate_m1_policy_at(
        &Decision::Allow,
        &verified_m1_with_static(static_extraction(
            M4_LARGE_TARBALL_WARNING_BYTES + 1,
            1,
            &[],
        )),
        parse_rfc3339_utc_seconds("2026-07-01T12:00:00Z").unwrap(),
    );

    assert_eq!(policy.decision, PolicyDecision::Ask);
    assert!(policy.reasons.contains(&PolicyReason::LargePackage));
    assert!(policy.rule_ids.contains(&PolicyRuleId::LargePackage));
    assert_eq!(
        policy.findings[0].observed,
        format!("{} bytes", M4_LARGE_TARBALL_WARNING_BYTES + 1)
    );
}

#[test]
/// Large verified file counts produce a structured policy warning.
fn large_file_count_threshold_produces_structured_warning() {
    let policy = evaluate_m1_policy_at(
        &Decision::Allow,
        &verified_m1_with_static(static_extraction(1, M4_LARGE_FILE_COUNT_WARNING + 1, &[])),
        parse_rfc3339_utc_seconds("2026-07-01T12:00:00Z").unwrap(),
    );

    assert_eq!(policy.decision, PolicyDecision::Ask);
    assert!(policy.reasons.contains(&PolicyReason::LargeFileCount));
    assert!(policy.rule_ids.contains(&PolicyRuleId::LargeFileCount));
    assert_eq!(
        policy.findings[0].observed,
        format!("{} files", M4_LARGE_FILE_COUNT_WARNING + 1)
    );
}

#[test]
/// Lifecycle scripts produce an ask warning before execution semantics.
fn lifecycle_threshold_produces_ask_warning() {
    let policy = evaluate_m1_policy_at(
        &Decision::Allow,
        &verified_m1_with_static(static_extraction(1, 1, &["postinstall"])),
        parse_rfc3339_utc_seconds("2026-07-01T12:00:00Z").unwrap(),
    );

    assert_eq!(policy.decision, PolicyDecision::Ask);
    assert!(policy
        .reasons
        .contains(&PolicyReason::LifecycleScriptPresent));
    assert!(policy.rule_ids.contains(&PolicyRuleId::LifecycleScript));
    assert_eq!(policy.findings[0].observed, "postinstall");
    assert_eq!(policy.findings[0].threshold, "no lifecycle scripts");
}

#[test]
/// Clean verified allow evidence remains allow with no threshold findings.
fn clean_verified_allow_has_no_policy_findings() {
    let policy = evaluate_m1_policy_at(
        &Decision::Allow,
        &verified_m1_with_static(static_extraction(1, 1, &[])),
        parse_rfc3339_utc_seconds("2026-07-01T12:00:00Z").unwrap(),
    );

    assert_eq!(policy.decision, PolicyDecision::Allow);
    assert_eq!(policy.required_next_action, PolicyNextAction::None);
    assert_eq!(policy.reasons, vec![PolicyReason::CallerRequestedAllow]);
    assert_eq!(policy.rule_ids, vec![PolicyRuleId::CallerRecommendation]);
    assert!(policy.findings.is_empty());
}

#[test]
/// Threshold boundary fixtures stay negative unless they exceed M4 limits.
fn threshold_boundaries_do_not_warn() {
    let now = parse_rfc3339_utc_seconds("2026-07-01T12:00:00Z").unwrap();
    let older_publish = evaluate_m1_policy_at(
        &Decision::Allow,
        &verified_m1_with_publish_time("2026-06-30T12:00:00.000Z"),
        now,
    );
    let boundary_static = evaluate_m1_policy_at(
        &Decision::Allow,
        &verified_m1_with_static(static_extraction(
            M4_LARGE_TARBALL_WARNING_BYTES,
            M4_LARGE_FILE_COUNT_WARNING,
            &[],
        )),
        now,
    );

    for policy in [older_publish, boundary_static] {
        assert_eq!(policy.decision, PolicyDecision::Allow);
        assert_eq!(policy.required_next_action, PolicyNextAction::None);
        assert!(policy.findings.is_empty());
    }
}

#[test]
/// Threshold warnings cannot weaken an explicit caller denial.
fn threshold_warnings_do_not_weaken_caller_deny() {
    let policy = evaluate_m1_policy_at(
        &Decision::Deny,
        &verified_m1_with_static(static_extraction(
            M4_LARGE_TARBALL_WARNING_BYTES + 1,
            M4_LARGE_FILE_COUNT_WARNING + 1,
            &["postinstall"],
        )),
        parse_rfc3339_utc_seconds("2026-07-01T12:00:00Z").unwrap(),
    );

    assert_eq!(policy.decision, PolicyDecision::Deny);
    assert_eq!(policy.required_next_action, PolicyNextAction::None);
    assert!(policy.reasons.contains(&PolicyReason::CallerRequestedDeny));
    assert!(policy.reasons.contains(&PolicyReason::LargePackage));
    assert!(policy.reasons.contains(&PolicyReason::LargeFileCount));
    assert!(policy
        .reasons
        .contains(&PolicyReason::LifecycleScriptPresent));
    assert_eq!(policy.findings.len(), 3);
}

#[test]
/// The freshness parser rejects impossible dates and accepts leap days.
fn recent_publish_parser_rejects_invalid_calendar_dates() {
    assert_eq!(parse_rfc3339_utc_seconds("2026-02-31T12:00:00Z"), None);
    assert!(parse_rfc3339_utc_seconds("2024-02-29T12:00:00Z").is_some());
}

#[test]
/// Unsupported and malformed M1 inputs map to retryable unsupported policy.
fn unsupported_and_malformed_input_map_to_unsupported_policy() {
    for reason in [M1Reason::UnsupportedSpec, M1Reason::MalformedSpec] {
        let policy = evaluate_m1_policy(
            &Decision::Deny,
            &M1Evidence::NoDownload {
                reason,
                downloaded: false,
            },
        );

        assert_eq!(policy.decision, PolicyDecision::Unsupported);
        assert_eq!(
            policy.required_next_action,
            PolicyNextAction::RetryNarrowerCommand
        );
    }
}

#[test]
/// Integrity mismatches are proof failures and map to deny.
fn integrity_mismatch_is_a_deny_policy() {
    let policy = evaluate_m1_policy(
        &Decision::Ask,
        &M1Evidence::Failed {
            reason: M1Reason::IntegrityMismatch,
            downloaded: true,
            detail: "sha512 mismatch".to_string(),
        },
    );

    assert_eq!(policy.decision, PolicyDecision::Deny);
    assert_eq!(policy.reasons, vec![PolicyReason::IntegrityMismatch]);
    assert_eq!(policy.rule_ids, vec![PolicyRuleId::IntegrityMismatch]);
}

#[test]
/// Registry failures map to inspection-error policy.
fn registry_failures_are_inspection_errors() {
    for reason in [
        M1Reason::RegistryError,
        M1Reason::MissingPackage,
        M1Reason::MissingVersion,
    ] {
        let policy = evaluate_m1_policy(
            &Decision::Allow,
            &M1Evidence::Failed {
                reason,
                downloaded: false,
                detail: "registry failure".to_string(),
            },
        );

        assert_eq!(policy.decision, PolicyDecision::InspectionError);
        assert_eq!(policy.required_next_action, PolicyNextAction::InspectOnly);
        assert_eq!(policy.rule_ids, vec![PolicyRuleId::InspectionUnavailable]);
    }
}

#[test]
/// Unsupported closure preserves execution-refused semantics.
fn m2_unsupported_closure_is_execution_refused_policy() {
    let policy = evaluate_m2_policy(&[M2Reason::UnsupportedClosure]);

    assert_eq!(policy.decision, PolicyDecision::ExecutionRefused);
    assert_eq!(policy.reasons, vec![PolicyReason::UnsupportedClosure]);
    assert_eq!(policy.required_next_action, PolicyNextAction::InspectOnly);
    assert_eq!(policy.rule_ids, vec![PolicyRuleId::UnsupportedClosure]);
}

#[test]
/// Every M4 decision has a documented default next action.
fn policy_decisions_have_default_next_actions() {
    let cases = [
        (PolicyDecision::Allow, PolicyNextAction::None),
        (PolicyDecision::Ask, PolicyNextAction::AskUser),
        (PolicyDecision::Deny, PolicyNextAction::None),
        (
            PolicyDecision::Unsupported,
            PolicyNextAction::RetryNarrowerCommand,
        ),
        (
            PolicyDecision::InspectionError,
            PolicyNextAction::InspectOnly,
        ),
        (
            PolicyDecision::ExecutionRefused,
            PolicyNextAction::InspectOnly,
        ),
    ];

    for (decision, next_action) in cases {
        assert_eq!(default_next_action_for_decision(&decision), next_action);
    }
}

#[test]
/// Representative reasons preserve stable next-action semantics.
fn reason_specific_next_actions_are_stable() {
    let unsupported = evaluate_m1_policy(
        &Decision::Deny,
        &M1Evidence::NoDownload {
            reason: M1Reason::UnsupportedSpec,
            downloaded: false,
        },
    );
    assert_eq!(
        unsupported.required_next_action,
        PolicyNextAction::RetryNarrowerCommand
    );

    let integrity = evaluate_m1_policy(
        &Decision::Ask,
        &M1Evidence::Failed {
            reason: M1Reason::IntegrityMismatch,
            downloaded: true,
            detail: "sha512 mismatch".to_string(),
        },
    );
    assert_eq!(integrity.required_next_action, PolicyNextAction::None);

    let inspection_error = evaluate_m1_policy(
        &Decision::Allow,
        &M1Evidence::Failed {
            reason: M1Reason::RegistryError,
            downloaded: false,
            detail: "registry unavailable".to_string(),
        },
    );
    assert_eq!(
        inspection_error.required_next_action,
        PolicyNextAction::InspectOnly
    );

    for reason in [
        M2Reason::LifecycleScriptPresent,
        M2Reason::UnsupportedClosure,
    ] {
        let policy = evaluate_m2_policy(&[reason]);
        assert_eq!(policy.required_next_action, PolicyNextAction::InspectOnly);
    }
}

#[test]
/// Missing or ambiguous bin selection remains retryable unsupported input.
fn m2_resolver_ambiguity_is_unsupported_policy() {
    for reason in [M2Reason::AmbiguousBin, M2Reason::MissingBin] {
        let policy = evaluate_m2_policy(&[reason]);

        assert_eq!(policy.decision, PolicyDecision::Unsupported);
        assert_eq!(
            policy.required_next_action,
            PolicyNextAction::RetryNarrowerCommand
        );
        assert_eq!(policy.rule_ids, vec![PolicyRuleId::ResolverAmbiguity]);
    }
}

#[test]
/// Non-interactive stop keeps legacy execution-refused decision for #66.
fn m2_non_interactive_stop_preserves_legacy_execution_refusal_policy() {
    let policy = evaluate_m2_policy(&[M2Reason::NonInteractiveStop]);

    assert_eq!(policy.decision, PolicyDecision::ExecutionRefused);
    assert_eq!(policy.reasons, vec![PolicyReason::NonInteractiveStop]);
    assert_eq!(policy.required_next_action, PolicyNextAction::AskUser);
    assert_eq!(policy.rule_ids, vec![PolicyRuleId::NonInteractiveStop]);
}

/// Build a verified M1 fixture without optional threshold evidence.
fn verified_m1() -> M1Evidence {
    verified_m1_with(None, None)
}

/// Build a verified M1 fixture with a publish timestamp.
fn verified_m1_with_publish_time(publish_time: &str) -> M1Evidence {
    verified_m1_with(Some(publish_time.to_string()), None)
}

/// Build a verified M1 fixture with static extraction evidence.
fn verified_m1_with_static(static_extraction: StaticExtractionEvidence) -> M1Evidence {
    verified_m1_with(None, Some(static_extraction))
}

/// Build a verified M1 fixture from optional registry and package facts.
fn verified_m1_with(
    publish_time: Option<String>,
    static_extraction: Option<StaticExtractionEvidence>,
) -> M1Evidence {
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
        artifact_identity: ArtifactIdentity {
            name: "create-example".to_string(),
            version: "1.2.3".to_string(),
            integrity: "sha512-example".to_string(),
            digest_algorithm: "sha512".to_string(),
            digest: "abc123".to_string(),
        },
        registry_evidence: RegistryEvidence {
            registry: registry_source(),
            package_scope: PackageScopeCategory::Unscoped,
            name: "create-example".to_string(),
            version: "1.2.3".to_string(),
            publish_time,
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
        static_extraction,
    }
}

/// Build static extraction evidence for policy threshold tests.
fn static_extraction(
    artifact_size_bytes: usize,
    file_count: usize,
    lifecycle_script_names: &[&str],
) -> StaticExtractionEvidence {
    let lifecycle_scripts = lifecycle_script_names
        .iter()
        .map(|name| ((*name).to_string(), "node script.js".to_string()))
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
        url: "https://registry.npmjs.org/".to_string(),
        scope: None,
    }
}
