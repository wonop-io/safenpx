//! M3 inspect JSON schema v0 for agents and CI.

use crate::m2_report::{closure_decision_name, required_next_action_name};
use crate::{
    build_authority_context, build_inspect_decision_receipt, evaluate_m1_policy,
    exit_code_for_report, ArtifactIdentity, AuthorityContext, CommandIntent, Decision,
    ExecutionReport, InspectDecisionReceipt, InspectFacts, InspectHeuristic, InspectModel,
    M1Evidence, M2ExecutionRefusalReport, PolicyDecision, PolicyNextAction, SourceContext,
};
use serde::Serialize;

/// M3 inspect JSON schema v0 emitted for agents and CI.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InspectJsonReport {
    /// Legacy package spec field retained during the 0.1 schema transition.
    #[serde(serialize_with = "crate::serialize_redacted_string")]
    pub package_spec: String,
    /// Legacy recommendation field retained during the 0.1 schema transition.
    pub recommendation: Decision,
    /// Legacy status marker retained during the 0.1 schema transition.
    pub status: &'static str,
    /// Legacy boundary note retained during the 0.1 schema transition.
    pub note: &'static str,
    /// Schema version for the M3 inspect JSON contract.
    pub schema_version: &'static str,
    /// Verified artifact identity when available.
    pub artifact: Option<ArtifactIdentity>,
    /// Parsed command intent before any execution.
    pub command_intent: CommandIntent,
    /// Caller-declared source context.
    pub source_context: SourceContext,
    /// Redacted authority context.
    pub authority_context: AuthorityContext,
    /// Evidence facts gathered before package code can run.
    pub facts: InspectFacts,
    /// Report-only heuristic signals.
    pub heuristics: Vec<InspectHeuristic>,
    /// Reserved for future hosted external evidence.
    pub external_evidence: Option<serde_json::Value>,
    /// Reserved for future attestations.
    pub attestations: Option<serde_json::Value>,
    /// Reserved for future release diff evidence.
    pub release_diff: Option<serde_json::Value>,
    /// Non-authoritative local/shareable inspect decision receipt.
    pub decision_receipt: Option<InspectDecisionReceipt>,
    /// Stable top-level decision vocabulary.
    pub decision: InspectJsonDecision,
    /// Stable machine-readable decision reasons.
    pub reasons: Vec<String>,
    /// Next action for agents and CI.
    pub required_next_action: InspectJsonNextAction,
    /// Execution evidence; null for inspect mode.
    pub execution: Option<ExecutionReport>,
    /// Process exit code implied by the report.
    pub exit_code: i32,
    /// Legacy shared inspect model retained during the 0.1 schema transition.
    pub inspect: InspectModel,
    /// Legacy M1 evidence retained during the 0.1 schema transition.
    pub m1: M1Evidence,
}

/// Stable M3 JSON decision vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectJsonDecision {
    /// Inspection found no policy reason to stop.
    Allow,
    /// Ask a human before execution.
    Ask,
    /// Deny because a proof failure or unsafe condition was found.
    Deny,
    /// The command shape is unsupported.
    Unsupported,
    /// Inspection failed before a trustworthy decision could be made.
    InspectionError,
    /// Execution was refused before package code ran.
    ExecutionRefused,
}

/// Stable M3 JSON next-action vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectJsonNextAction {
    /// No next action is required.
    None,
    /// Ask a human before proceeding.
    AskUser,
    /// Retry with a narrower command shape.
    RetryNarrowerCommand,
    /// Use inspect-only output.
    InspectOnly,
    /// Use a future explicit override path.
    ExplicitOverride,
    /// The requested command shape is unsupported.
    Unsupported,
}

/// Build the M3 inspect JSON schema from an internal report.
pub fn build_inspect_json_report(report: &crate::Report) -> InspectJsonReport {
    let policy = evaluate_m1_policy(&report.recommendation, &report.m1);
    let decision = inspect_json_decision_for_policy(&policy.decision);
    let required_next_action = inspect_json_next_action_for_policy(&policy.required_next_action);
    let exit_code = exit_code_for_report(report);

    InspectJsonReport {
        package_spec: report.package_spec.clone(),
        recommendation: report.recommendation.clone(),
        status: report.status,
        note: report.note,
        schema_version: "0.1",
        artifact: report.inspect.facts.artifact.clone(),
        command_intent: report.inspect.facts.command.clone(),
        source_context: report
            .inspect
            .authority_context
            .redacted
            .source_context
            .clone(),
        authority_context: report.inspect.authority_context.redacted.clone(),
        facts: report.inspect.facts.clone(),
        heuristics: report.inspect.heuristics.clone(),
        external_evidence: None,
        attestations: None,
        release_diff: None,
        decision_receipt: Some(build_inspect_decision_receipt(
            report,
            decision.clone(),
            required_next_action.clone(),
            exit_code,
        )),
        decision,
        reasons: report.inspect.decision.reasons.clone(),
        required_next_action,
        execution: None,
        exit_code,
        inspect: report.inspect.clone(),
        m1: report.m1.clone(),
    }
}

/// Build the M3 JSON schema for an execution-refused path.
pub fn build_m2_execution_refusal_json_report(
    report: &M2ExecutionRefusalReport,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "0.1",
        "artifact": null,
        "command_intent": report.command,
        "source_context": SourceContext::Unknown,
        "authority_context": build_authority_context(
            &report.command.requested,
            &SourceContext::Unknown,
            None,
            None,
        ),
        "facts": null,
        "heuristics": [],
        "external_evidence": null,
        "attestations": null,
        "release_diff": null,
        "decision_receipt": null,
        "decision": closure_decision_name(&report.decision),
        "reasons": report.reasons,
        "required_next_action": required_next_action_name(&report.required_next_action),
        "execution": null,
        "exit_code": report.exit_code,
        "command": report.command,
    })
}

/// Map the canonical M4 policy decision to the public M3 JSON decision.
fn inspect_json_decision_for_policy(decision: &PolicyDecision) -> InspectJsonDecision {
    match decision {
        PolicyDecision::Allow => InspectJsonDecision::Allow,
        PolicyDecision::Ask => InspectJsonDecision::Ask,
        PolicyDecision::Deny => InspectJsonDecision::Deny,
        PolicyDecision::Unsupported => InspectJsonDecision::Unsupported,
        PolicyDecision::InspectionError => InspectJsonDecision::InspectionError,
        PolicyDecision::ExecutionRefused => InspectJsonDecision::ExecutionRefused,
    }
}

/// Map the canonical M4 next action to the public M3 JSON next action.
fn inspect_json_next_action_for_policy(action: &PolicyNextAction) -> InspectJsonNextAction {
    match action {
        PolicyNextAction::None => InspectJsonNextAction::None,
        PolicyNextAction::AskUser => InspectJsonNextAction::AskUser,
        PolicyNextAction::RetryNarrowerCommand => InspectJsonNextAction::RetryNarrowerCommand,
        PolicyNextAction::InspectOnly => InspectJsonNextAction::InspectOnly,
        PolicyNextAction::ExplicitOverride => InspectJsonNextAction::ExplicitOverride,
        PolicyNextAction::Unsupported => InspectJsonNextAction::Unsupported,
    }
}
