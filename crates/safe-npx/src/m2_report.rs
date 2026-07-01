//! M2 execution-refusal report helpers.

use crate::{
    evaluate_m2_policy, exit_code_for_policy_decision, ClosureDecision, M2Reason, PolicyDecision,
    PolicyNextAction, RequiredNextAction,
};

/// Return the next action implied by M2 refusal reasons.
pub(crate) fn required_next_action_for_m2_reasons(reasons: &[M2Reason]) -> RequiredNextAction {
    match evaluate_m2_policy(reasons).required_next_action {
        PolicyNextAction::None => RequiredNextAction::None,
        PolicyNextAction::AskUser => RequiredNextAction::AskUser,
        PolicyNextAction::RetryNarrowerCommand => RequiredNextAction::RetryNarrowerCommand,
        PolicyNextAction::InspectOnly => RequiredNextAction::InspectOnly,
        PolicyNextAction::ExplicitOverride => RequiredNextAction::ExplicitOverride,
        PolicyNextAction::Unsupported => RequiredNextAction::Unsupported,
    }
}

/// Return the completed-proof decision semantics for a set of M2 reasons.
pub(crate) fn closure_decision_for_m2_reasons(reasons: &[M2Reason]) -> ClosureDecision {
    match evaluate_m2_policy(reasons).decision {
        PolicyDecision::Allow => ClosureDecision::Allow,
        PolicyDecision::Ask => ClosureDecision::Ask,
        PolicyDecision::Deny => ClosureDecision::Deny,
        PolicyDecision::Unsupported => ClosureDecision::Unsupported,
        PolicyDecision::InspectionError => ClosureDecision::InspectionError,
        PolicyDecision::ExecutionRefused => ClosureDecision::ExecutionRefused,
    }
}

/// Return the M2 fixture exit code for a closure decision.
pub(crate) fn exit_code_for_closure_decision(decision: &ClosureDecision) -> i32 {
    exit_code_for_policy_decision(&policy_decision_for_closure_decision(decision))
}

/// Convert M2 closure decisions back to the canonical M4 policy vocabulary.
fn policy_decision_for_closure_decision(decision: &ClosureDecision) -> PolicyDecision {
    match decision {
        ClosureDecision::Allow => PolicyDecision::Allow,
        ClosureDecision::Ask => PolicyDecision::Ask,
        ClosureDecision::Deny => PolicyDecision::Deny,
        ClosureDecision::Unsupported => PolicyDecision::Unsupported,
        ClosureDecision::InspectionError => PolicyDecision::InspectionError,
        ClosureDecision::ExecutionRefused => PolicyDecision::ExecutionRefused,
    }
}

/// Format M2 reasons as stable comma-separated names for terminal output.
pub(crate) fn format_m2_reasons(reasons: &[M2Reason]) -> String {
    reasons
        .iter()
        .map(m2_reason_name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Return the stable serialized name for an M2 decision.
pub(crate) fn closure_decision_name(decision: &ClosureDecision) -> &'static str {
    match decision {
        ClosureDecision::Allow => "allow",
        ClosureDecision::Ask => "ask",
        ClosureDecision::Deny => "deny",
        ClosureDecision::Unsupported => "unsupported",
        ClosureDecision::InspectionError => "inspection_error",
        ClosureDecision::ExecutionRefused => "execution_refused",
    }
}

/// Return the stable serialized name for a next action.
pub(crate) fn required_next_action_name(action: &RequiredNextAction) -> &'static str {
    match action {
        RequiredNextAction::None => "none",
        RequiredNextAction::AskUser => "ask_user",
        RequiredNextAction::RetryNarrowerCommand => "retry_narrower_command",
        RequiredNextAction::InspectOnly => "inspect_only",
        RequiredNextAction::ExplicitOverride => "explicit_override",
        RequiredNextAction::Unsupported => "unsupported",
    }
}

/// Return the stable serialized name for an M2 reason.
fn m2_reason_name(reason: &M2Reason) -> &'static str {
    match reason {
        M2Reason::InteractiveApprovalRequired => "interactive_approval_required",
        M2Reason::AmbiguousBin => "ambiguous_bin",
        M2Reason::MissingBin => "missing_bin",
        M2Reason::LifecycleScriptPresent => "lifecycle_script_present",
        M2Reason::UnsupportedClosure => "unsupported_closure",
        M2Reason::MetadataChanged => "metadata_changed",
        M2Reason::CacheIdentityMismatch => "cache_identity_mismatch",
        M2Reason::RegistryPrecedenceMismatch => "registry_precedence_mismatch",
        M2Reason::ShimIdentityMismatch => "shim_identity_mismatch",
        M2Reason::NonInteractiveStop => "non_interactive_stop",
    }
}
