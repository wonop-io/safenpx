//! M4 provisional policy evaluation model.
//!
//! Policy evaluation sits between collected evidence and renderers. Later M4
//! work adds thresholds and exit-code semantics here instead of spreading them
//! through human output, JSON output, and CLI plumbing.

use crate::{Decision, M1Evidence, M1Reason, M2Reason};
use serde::Serialize;

/// Current provisional policy version.
pub const M4_POLICY_VERSION: &str = "m4-policy-v0";

/// Versioned policy result consumed by reports, JSON, and exit-code work.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PolicyEvaluation {
    /// Provisional policy version that produced this result.
    pub policy_version: &'static str,
    /// Canonical M4 decision.
    pub decision: PolicyDecision,
    /// Stable machine-readable reasons supporting the decision.
    pub reasons: Vec<PolicyReason>,
    /// Required next action for humans, agents, and CI.
    pub required_next_action: PolicyNextAction,
    /// Policy rule identifiers that fired.
    pub rule_ids: Vec<PolicyRuleId>,
}

impl PolicyEvaluation {
    /// Build a policy evaluation with the repository's current policy version.
    pub fn new(
        decision: PolicyDecision,
        reasons: Vec<PolicyReason>,
        required_next_action: PolicyNextAction,
        rule_ids: Vec<PolicyRuleId>,
    ) -> Self {
        Self {
            policy_version: M4_POLICY_VERSION,
            decision,
            reasons,
            required_next_action,
            rule_ids,
        }
    }
}

/// Canonical M4 decision vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecision {
    /// Evidence and policy permit the requested action.
    Allow,
    /// A human decision is required before execution.
    Ask,
    /// A proof failure or known unsafe condition was found.
    Deny,
    /// The requested input or command shape is unsupported.
    Unsupported,
    /// Evidence could not be collected reliably.
    InspectionError,
    /// Inspection succeeded, but execution closure cannot be proven.
    ExecutionRefused,
}

/// Agent-readable next-action vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyNextAction {
    /// No action is required.
    None,
    /// Ask a human before proceeding.
    AskUser,
    /// Retry with a narrower exact command.
    RetryNarrowerCommand,
    /// Use inspect-only output; execution is not available.
    InspectOnly,
    /// Use a future explicit override path.
    ExplicitOverride,
    /// The requested command shape is unsupported.
    Unsupported,
}

/// Stable policy reasons.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyReason {
    /// User or caller requested allow.
    CallerRequestedAllow,
    /// User or caller requested ask.
    CallerRequestedAsk,
    /// User or caller requested deny.
    CallerRequestedDeny,
    /// The package spec is unsupported.
    UnsupportedSpec,
    /// The package spec is malformed.
    MalformedSpec,
    /// Registry metadata could not be fetched or interpreted.
    RegistryError,
    /// Downloaded bytes failed integrity verification.
    IntegrityMismatch,
    /// The package name does not exist in the selected registry.
    MissingPackage,
    /// The package version does not exist in the selected registry.
    MissingVersion,
    /// More than one binary could match the command.
    AmbiguousBin,
    /// No executable binary could be selected.
    MissingBin,
    /// A lifecycle script is present in executable package metadata.
    LifecycleScriptPresent,
    /// Full execution closure cannot be proven.
    UnsupportedClosure,
    /// Metadata changed between inspection and execution preparation.
    MetadataChanged,
    /// Cache entry identity does not match inspected evidence.
    CacheIdentityMismatch,
    /// Registry source selected for execution differs from inspection.
    RegistryPrecedenceMismatch,
    /// Generated shim identity does not match deterministic evidence.
    ShimIdentityMismatch,
    /// Non-interactive context requires stopping instead of prompting.
    NonInteractiveStop,
}

/// Stable identifiers for policy rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyRuleId {
    /// Verified evidence currently follows the caller-selected recommendation.
    CallerRecommendation,
    /// Unsupported input fails closed before network or execution work.
    UnsupportedInput,
    /// Malformed input fails closed before network or execution work.
    MalformedInput,
    /// Integrity mismatch is a proof failure.
    IntegrityMismatch,
    /// Registry or extraction evidence is unavailable.
    InspectionUnavailable,
    /// Bin selection is ambiguous or missing.
    ResolverAmbiguity,
    /// Execution closure cannot be proven.
    UnsupportedClosure,
    /// Static lifecycle evidence blocks execution.
    LifecycleScript,
    /// Race or identity evidence changed after inspection.
    IdentityDrift,
    /// Non-interactive mode must stop before prompting.
    NonInteractiveStop,
}

/// Evaluate M1/M3 report evidence into the canonical M4 policy result.
pub fn evaluate_m1_policy(recommendation: &Decision, m1: &M1Evidence) -> PolicyEvaluation {
    match m1 {
        M1Evidence::Verified { .. } => {
            let (decision, reason, next_action) = match recommendation {
                Decision::Allow => (
                    PolicyDecision::Allow,
                    PolicyReason::CallerRequestedAllow,
                    PolicyNextAction::None,
                ),
                Decision::Ask => (
                    PolicyDecision::Ask,
                    PolicyReason::CallerRequestedAsk,
                    PolicyNextAction::AskUser,
                ),
                Decision::Deny => (
                    PolicyDecision::Deny,
                    PolicyReason::CallerRequestedDeny,
                    PolicyNextAction::None,
                ),
            };
            PolicyEvaluation::new(
                decision,
                vec![reason],
                next_action,
                vec![PolicyRuleId::CallerRecommendation],
            )
        }
        M1Evidence::NoDownload { reason, .. } => match reason {
            M1Reason::UnsupportedSpec => PolicyEvaluation::new(
                PolicyDecision::Unsupported,
                vec![PolicyReason::UnsupportedSpec],
                PolicyNextAction::RetryNarrowerCommand,
                vec![PolicyRuleId::UnsupportedInput],
            ),
            M1Reason::MalformedSpec => PolicyEvaluation::new(
                PolicyDecision::Unsupported,
                vec![PolicyReason::MalformedSpec],
                PolicyNextAction::RetryNarrowerCommand,
                vec![PolicyRuleId::MalformedInput],
            ),
            other => failed_m1_policy(other),
        },
        M1Evidence::Failed { reason, .. } => failed_m1_policy(reason),
    }
}

/// Evaluate M2 execution-closure reasons into the canonical M4 policy result.
pub fn evaluate_m2_policy(reasons: &[M2Reason]) -> PolicyEvaluation {
    let decision = m2_policy_decision(reasons);
    let required_next_action =
        if reasons.contains(&M2Reason::AmbiguousBin) || reasons.contains(&M2Reason::MissingBin) {
            PolicyNextAction::RetryNarrowerCommand
        } else if reasons.contains(&M2Reason::NonInteractiveStop) {
            PolicyNextAction::AskUser
        } else if reasons.contains(&M2Reason::UnsupportedClosure) {
            PolicyNextAction::InspectOnly
        } else {
            PolicyNextAction::Unsupported
        };

    PolicyEvaluation::new(
        decision,
        reasons.iter().map(policy_reason_for_m2_reason).collect(),
        required_next_action,
        reasons.iter().map(policy_rule_for_m2_reason).collect(),
    )
}

fn m2_policy_decision(reasons: &[M2Reason]) -> PolicyDecision {
    if reasons.iter().any(|reason| {
        matches!(
            reason,
            M2Reason::UnsupportedClosure
                | M2Reason::LifecycleScriptPresent
                | M2Reason::MetadataChanged
                | M2Reason::CacheIdentityMismatch
                | M2Reason::RegistryPrecedenceMismatch
                | M2Reason::ShimIdentityMismatch
                | M2Reason::NonInteractiveStop
        )
    }) {
        return PolicyDecision::ExecutionRefused;
    }
    if reasons
        .iter()
        .any(|reason| matches!(reason, M2Reason::AmbiguousBin | M2Reason::MissingBin))
    {
        return PolicyDecision::Unsupported;
    }

    PolicyDecision::Ask
}

fn failed_m1_policy(reason: &M1Reason) -> PolicyEvaluation {
    match reason {
        M1Reason::IntegrityMismatch => PolicyEvaluation::new(
            PolicyDecision::Deny,
            vec![PolicyReason::IntegrityMismatch],
            PolicyNextAction::None,
            vec![PolicyRuleId::IntegrityMismatch],
        ),
        M1Reason::UnsupportedSpec => PolicyEvaluation::new(
            PolicyDecision::Unsupported,
            vec![PolicyReason::UnsupportedSpec],
            PolicyNextAction::RetryNarrowerCommand,
            vec![PolicyRuleId::UnsupportedInput],
        ),
        M1Reason::MalformedSpec => PolicyEvaluation::new(
            PolicyDecision::Unsupported,
            vec![PolicyReason::MalformedSpec],
            PolicyNextAction::RetryNarrowerCommand,
            vec![PolicyRuleId::MalformedInput],
        ),
        M1Reason::RegistryError => PolicyEvaluation::new(
            PolicyDecision::InspectionError,
            vec![PolicyReason::RegistryError],
            PolicyNextAction::InspectOnly,
            vec![PolicyRuleId::InspectionUnavailable],
        ),
        M1Reason::MissingPackage => PolicyEvaluation::new(
            PolicyDecision::InspectionError,
            vec![PolicyReason::MissingPackage],
            PolicyNextAction::InspectOnly,
            vec![PolicyRuleId::InspectionUnavailable],
        ),
        M1Reason::MissingVersion => PolicyEvaluation::new(
            PolicyDecision::InspectionError,
            vec![PolicyReason::MissingVersion],
            PolicyNextAction::InspectOnly,
            vec![PolicyRuleId::InspectionUnavailable],
        ),
    }
}

fn policy_reason_for_m2_reason(reason: &M2Reason) -> PolicyReason {
    match reason {
        M2Reason::InteractiveApprovalRequired => PolicyReason::CallerRequestedAsk,
        M2Reason::AmbiguousBin => PolicyReason::AmbiguousBin,
        M2Reason::MissingBin => PolicyReason::MissingBin,
        M2Reason::LifecycleScriptPresent => PolicyReason::LifecycleScriptPresent,
        M2Reason::UnsupportedClosure => PolicyReason::UnsupportedClosure,
        M2Reason::MetadataChanged => PolicyReason::MetadataChanged,
        M2Reason::CacheIdentityMismatch => PolicyReason::CacheIdentityMismatch,
        M2Reason::RegistryPrecedenceMismatch => PolicyReason::RegistryPrecedenceMismatch,
        M2Reason::ShimIdentityMismatch => PolicyReason::ShimIdentityMismatch,
        M2Reason::NonInteractiveStop => PolicyReason::NonInteractiveStop,
    }
}

fn policy_rule_for_m2_reason(reason: &M2Reason) -> PolicyRuleId {
    match reason {
        M2Reason::InteractiveApprovalRequired => PolicyRuleId::CallerRecommendation,
        M2Reason::AmbiguousBin | M2Reason::MissingBin => PolicyRuleId::ResolverAmbiguity,
        M2Reason::LifecycleScriptPresent => PolicyRuleId::LifecycleScript,
        M2Reason::UnsupportedClosure => PolicyRuleId::UnsupportedClosure,
        M2Reason::MetadataChanged
        | M2Reason::CacheIdentityMismatch
        | M2Reason::RegistryPrecedenceMismatch
        | M2Reason::ShimIdentityMismatch => PolicyRuleId::IdentityDrift,
        M2Reason::NonInteractiveStop => PolicyRuleId::NonInteractiveStop,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ArtifactIdentity, PackageScopeCategory, RegistryEvidence, RegistrySource, ResolvedPackage,
    };
    use std::collections::BTreeMap;

    #[test]
    fn verified_evidence_uses_caller_recommendation_policy() {
        let policy = evaluate_m1_policy(&Decision::Ask, &verified_m1());

        assert_eq!(policy.policy_version, M4_POLICY_VERSION);
        assert_eq!(policy.decision, PolicyDecision::Ask);
        assert_eq!(policy.reasons, vec![PolicyReason::CallerRequestedAsk]);
        assert_eq!(policy.required_next_action, PolicyNextAction::AskUser);
        assert_eq!(policy.rule_ids, vec![PolicyRuleId::CallerRecommendation]);
    }

    #[test]
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
    fn m2_unsupported_closure_is_execution_refused_policy() {
        let policy = evaluate_m2_policy(&[M2Reason::UnsupportedClosure]);

        assert_eq!(policy.decision, PolicyDecision::ExecutionRefused);
        assert_eq!(policy.reasons, vec![PolicyReason::UnsupportedClosure]);
        assert_eq!(policy.required_next_action, PolicyNextAction::InspectOnly);
        assert_eq!(policy.rule_ids, vec![PolicyRuleId::UnsupportedClosure]);
    }

    #[test]
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
    fn m2_non_interactive_stop_preserves_legacy_execution_refusal_policy() {
        let policy = evaluate_m2_policy(&[M2Reason::NonInteractiveStop]);

        assert_eq!(policy.decision, PolicyDecision::ExecutionRefused);
        assert_eq!(policy.reasons, vec![PolicyReason::NonInteractiveStop]);
        assert_eq!(policy.required_next_action, PolicyNextAction::AskUser);
        assert_eq!(policy.rule_ids, vec![PolicyRuleId::NonInteractiveStop]);
    }

    fn verified_m1() -> M1Evidence {
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
            static_extraction: None,
        }
    }

    fn registry_source() -> RegistrySource {
        RegistrySource {
            url: "https://registry.npmjs.org/".to_string(),
            scope: None,
        }
    }
}
