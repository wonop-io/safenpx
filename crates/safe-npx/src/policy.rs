//! M4 provisional policy evaluation model.
//!
//! Policy evaluation sits between collected evidence and renderers. Later M4
//! work adds thresholds and exit-code semantics here instead of spreading them
//! through human output, JSON output, and CLI plumbing.

use crate::policy_time::{current_unix_seconds, parse_rfc3339_utc_seconds};
use crate::{Decision, M1Evidence, M1Reason, M2Reason, StaticExtractionEvidence};
use serde::Serialize;

/// Current provisional policy version.
pub const M4_POLICY_VERSION: &str = "m4-policy-v0";
/// Recent publish warning threshold for package versions.
pub const M4_RECENT_PUBLISH_WARNING_HOURS: u64 = 24;
/// Large tarball warning threshold for verified root package bytes.
pub const M4_LARGE_TARBALL_WARNING_BYTES: usize = 5 * 1024 * 1024;
/// Large file-count warning threshold for verified root package contents.
pub const M4_LARGE_FILE_COUNT_WARNING: usize = 500;

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
    /// Structured evidence for threshold and policy findings.
    pub findings: Vec<PolicyFinding>,
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
            findings: Vec::new(),
        }
    }
}

/// Structured policy evidence for a fired rule.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PolicyFinding {
    /// Stable policy rule identifier.
    pub rule_id: PolicyRuleId,
    /// Stable reason associated with the rule.
    pub reason: PolicyReason,
    /// Observed value that triggered the rule.
    pub observed: String,
    /// Threshold or expected value for the rule.
    pub threshold: String,
    /// Human-readable evidence summary for reports and traces.
    pub evidence: String,
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
    /// Package version was published recently enough to require caution.
    RecentPublish,
    /// Verified root tarball is larger than the provisional threshold.
    LargePackage,
    /// Verified root tarball contains more files than the provisional threshold.
    LargeFileCount,
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
    /// Package version was published within the freshness warning window.
    RecentPublish,
    /// Verified tarball bytes exceed the provisional large-package threshold.
    LargePackage,
    /// Verified package file count exceeds the provisional threshold.
    LargeFileCount,
    /// Race or identity evidence changed after inspection.
    IdentityDrift,
    /// Non-interactive mode must stop before prompting.
    NonInteractiveStop,
}

/// Evaluate M1/M3 report evidence into the canonical M4 policy result.
pub fn evaluate_m1_policy(recommendation: &Decision, m1: &M1Evidence) -> PolicyEvaluation {
    evaluate_m1_policy_at(recommendation, m1, current_unix_seconds())
}

/// Evaluate M1/M3 report evidence at a deterministic unix timestamp.
pub fn evaluate_m1_policy_at(
    recommendation: &Decision,
    m1: &M1Evidence,
    now_unix_seconds: u64,
) -> PolicyEvaluation {
    match m1 {
        M1Evidence::Verified {
            registry_evidence,
            static_extraction,
            ..
        } => {
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
            let mut policy = PolicyEvaluation::new(
                decision,
                vec![reason],
                next_action,
                vec![PolicyRuleId::CallerRecommendation],
            );
            apply_threshold_findings(
                &mut policy,
                registry_evidence.publish_time.as_deref(),
                static_extraction.as_ref(),
                now_unix_seconds,
            );
            policy
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

fn apply_threshold_findings(
    policy: &mut PolicyEvaluation,
    publish_time: Option<&str>,
    static_extraction: Option<&StaticExtractionEvidence>,
    now_unix_seconds: u64,
) {
    if let Some(publish_time) = publish_time {
        if let Some(published_unix_seconds) = parse_rfc3339_utc_seconds(publish_time) {
            if published_unix_seconds >= now_unix_seconds
                || now_unix_seconds - published_unix_seconds
                    < M4_RECENT_PUBLISH_WARNING_HOURS * 60 * 60
            {
                let observed = if published_unix_seconds >= now_unix_seconds {
                    format!("published_at={publish_time}; clock_skew_or_future_publish=true")
                } else {
                    let age_hours = (now_unix_seconds - published_unix_seconds) / 60 / 60;
                    format!("published_at={publish_time}; age_hours={age_hours}")
                };
                push_threshold_warning(
                    policy,
                    PolicyRuleId::RecentPublish,
                    PolicyReason::RecentPublish,
                    observed,
                    format!("<{}h", M4_RECENT_PUBLISH_WARNING_HOURS),
                    "package version was published within the provisional freshness warning window"
                        .to_string(),
                );
            }
        }
    }

    if let Some(static_extraction) = static_extraction {
        if static_extraction.artifact_size_bytes > M4_LARGE_TARBALL_WARNING_BYTES {
            push_threshold_warning(
                policy,
                PolicyRuleId::LargePackage,
                PolicyReason::LargePackage,
                format!("{} bytes", static_extraction.artifact_size_bytes),
                format!(">{} bytes", M4_LARGE_TARBALL_WARNING_BYTES),
                "verified root tarball exceeds the provisional size warning threshold".to_string(),
            );
        }

        if static_extraction.file_count > M4_LARGE_FILE_COUNT_WARNING {
            push_threshold_warning(
                policy,
                PolicyRuleId::LargeFileCount,
                PolicyReason::LargeFileCount,
                format!("{} files", static_extraction.file_count),
                format!(">{} files", M4_LARGE_FILE_COUNT_WARNING),
                "verified root tarball exceeds the provisional file-count warning threshold"
                    .to_string(),
            );
        }

        if !static_extraction.metadata.lifecycle_scripts.is_empty() {
            let scripts = static_extraction
                .metadata
                .lifecycle_scripts
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            push_threshold_warning(
                policy,
                PolicyRuleId::LifecycleScript,
                PolicyReason::LifecycleScriptPresent,
                scripts,
                "no lifecycle scripts".to_string(),
                "root package declares lifecycle scripts that can execute during install"
                    .to_string(),
            );
        }
    }
}

/// Add a threshold warning to policy while preserving stronger decisions.
fn push_threshold_warning(
    policy: &mut PolicyEvaluation,
    rule_id: PolicyRuleId,
    reason: PolicyReason,
    observed: String,
    threshold: String,
    evidence: String,
) {
    if !policy.reasons.contains(&reason) {
        policy.reasons.push(reason.clone());
    }
    if !policy.rule_ids.contains(&rule_id) {
        policy.rule_ids.push(rule_id.clone());
    }
    policy.findings.push(PolicyFinding {
        rule_id,
        reason,
        observed,
        threshold,
        evidence,
    });

    if matches!(policy.decision, PolicyDecision::Allow) {
        policy.decision = PolicyDecision::Ask;
        policy.required_next_action = PolicyNextAction::AskUser;
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
        } else if matches!(decision, PolicyDecision::ExecutionRefused) {
            PolicyNextAction::InspectOnly
        } else {
            default_next_action_for_decision(&decision)
        };

    PolicyEvaluation::new(
        decision,
        reasons.iter().map(policy_reason_for_m2_reason).collect(),
        required_next_action,
        reasons.iter().map(policy_rule_for_m2_reason).collect(),
    )
}

/// Return the default next action for a canonical M4 policy decision.
pub fn default_next_action_for_decision(decision: &PolicyDecision) -> PolicyNextAction {
    match decision {
        PolicyDecision::Allow | PolicyDecision::Deny => PolicyNextAction::None,
        PolicyDecision::Ask => PolicyNextAction::AskUser,
        PolicyDecision::Unsupported => PolicyNextAction::RetryNarrowerCommand,
        PolicyDecision::InspectionError | PolicyDecision::ExecutionRefused => {
            PolicyNextAction::InspectOnly
        }
    }
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

/// Map failed M1 evidence into the canonical policy vocabulary.
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

/// Map an M2 refusal reason to a canonical policy reason.
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

/// Map an M2 refusal reason to a stable policy rule identifier.
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
#[path = "policy_tests.rs"]
mod policy_tests;
