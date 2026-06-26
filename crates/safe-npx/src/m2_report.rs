//! M2 execution-refusal report helpers.

use crate::{
    ClosureDecision, M2Reason, RequiredNextAction, M2_EXECUTION_REFUSED_EXIT_CODE,
    M2_UNSUPPORTED_EXIT_CODE,
};

/// Return the next action implied by M2 refusal reasons.
pub(crate) fn required_next_action_for_m2_reasons(reasons: &[M2Reason]) -> RequiredNextAction {
    if reasons.contains(&M2Reason::AmbiguousBin) || reasons.contains(&M2Reason::MissingBin) {
        return RequiredNextAction::RetryNarrowerCommand;
    }
    if reasons.contains(&M2Reason::NonInteractiveStop) {
        return RequiredNextAction::AskUser;
    }
    if reasons.contains(&M2Reason::UnsupportedClosure) {
        return RequiredNextAction::InspectOnly;
    }

    RequiredNextAction::Unsupported
}

/// Return the completed-proof decision semantics for a set of M2 reasons.
pub(crate) fn closure_decision_for_m2_reasons(reasons: &[M2Reason]) -> ClosureDecision {
    if reasons
        .iter()
        .any(|reason| reason.refusal_decision() == ClosureDecision::ExecutionRefused)
    {
        return ClosureDecision::ExecutionRefused;
    }
    if reasons
        .iter()
        .any(|reason| reason.refusal_decision() == ClosureDecision::Unsupported)
    {
        return ClosureDecision::Unsupported;
    }
    if reasons
        .iter()
        .any(|reason| reason.refusal_decision() == ClosureDecision::InspectionError)
    {
        return ClosureDecision::InspectionError;
    }
    if reasons
        .iter()
        .any(|reason| reason.refusal_decision() == ClosureDecision::Deny)
    {
        return ClosureDecision::Deny;
    }

    ClosureDecision::Ask
}

/// Return the M2 fixture exit code for a closure decision.
pub(crate) fn exit_code_for_closure_decision(decision: &ClosureDecision) -> i32 {
    match decision {
        ClosureDecision::Allow | ClosureDecision::Ask => 0,
        ClosureDecision::Unsupported => M2_UNSUPPORTED_EXIT_CODE,
        ClosureDecision::ExecutionRefused => M2_EXECUTION_REFUSED_EXIT_CODE,
        ClosureDecision::Deny => 1,
        ClosureDecision::InspectionError => 3,
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
