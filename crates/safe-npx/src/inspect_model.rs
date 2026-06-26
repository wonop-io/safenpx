//! Shared M3 inspect evidence model for human and JSON reports.

use crate::{
    ArtifactIdentity, CommandIntent, Decision, M1Reason, RegistryEvidence, RegistrySource,
    ResolvedPackage, SourceContext, StaticExtractionEvidence,
};
use serde::Serialize;

/// Shared M3 inspect model consumed by human and JSON renderers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InspectModel {
    /// Verified, declared, or failure facts gathered before execution.
    pub facts: InspectFacts,
    /// Report-only signals that do not become M3 hard policy denials.
    pub heuristics: Vec<InspectHeuristic>,
    /// Current decision summary and required next action.
    pub decision: InspectDecision,
    /// Initial authority context for this inspect report.
    pub authority_context: InspectAuthorityContext,
    /// Explicit execution state for the inspect report.
    pub execution_state: InspectExecutionState,
}

/// Facts gathered before package code can run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InspectFacts {
    /// Parsed command intent.
    pub command: CommandIntent,
    /// Resolved package coordinates when resolution succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_package: Option<ResolvedPackage>,
    /// Registry metadata evidence when resolution succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<RegistryEvidence>,
    /// Verified root artifact identity when integrity succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<ArtifactIdentity>,
    /// Verified root package extraction facts when inspect extraction succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_package: Option<StaticExtractionEvidence>,
    /// Refusal or failure fact for no-download and failed inspections.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<InspectRefusalFact>,
}

/// Refusal or failure fact captured before execution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InspectRefusalFact {
    /// Stable refusal state.
    pub state: InspectRefusalState,
    /// Stable M1 reason.
    pub reason: M1Reason,
    /// Whether registry or tarball bytes were downloaded.
    pub downloaded: bool,
    /// Optional failure detail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Refusal fact state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectRefusalState {
    /// Input was refused before registry or tarball access.
    NoDownload,
    /// Registry, download, integrity, or extraction failed before execution.
    Failed,
}

/// Report-only inspect heuristic.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InspectHeuristic {
    /// Stable heuristic kind.
    pub kind: InspectHeuristicKind,
    /// Evidence source for this heuristic.
    pub source: &'static str,
    /// Human-readable summary.
    pub message: String,
    /// Whether this heuristic is report-only in M3.
    pub report_only: bool,
}

/// Stable heuristic vocabulary for M3 inspect reports.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectHeuristicKind {
    /// Lifecycle scripts are present in root package metadata.
    LifecycleScriptsPresent,
    /// Dependency declarations are present but are not verified closure proof.
    DependencyDeclarationsPresent,
    /// Package shape is unusual but not a hard denial in M3.
    UnusualPackageShape,
}

/// Current inspect decision summary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InspectDecision {
    /// Current recommendation.
    pub recommendation: Decision,
    /// Stable textual reasons for the current recommendation.
    pub reasons: Vec<String>,
    /// Required next action for a human or agent.
    pub required_next_action: InspectNextAction,
}

/// Inspect next action vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectNextAction {
    /// Ask a human before execution.
    AskUser,
    /// Stop; no execution path is available for this report.
    Stop,
    /// Retry with a narrower exact package command.
    RetryNarrowerCommand,
}

/// Initial authority context for M3 inspect output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InspectAuthorityContext {
    /// Raw command/package intent.
    pub command_intent: String,
    /// Caller-declared source context, or unknown when undeclared.
    pub source_context: SourceContext,
    /// Selected registry source when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry_source: Option<RegistrySource>,
    /// Package scope category when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_scope: Option<String>,
}

/// Execution state for the inspect report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InspectExecutionState {
    /// Stable execution state.
    pub state: InspectExecutionStateKind,
    /// Whether package code ran.
    pub package_code_executed: bool,
}

/// Stable execution state vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectExecutionStateKind {
    /// Inspect completed and stopped before execution.
    StoppedBeforeExecution,
    /// Input was refused before execution.
    RefusedBeforeExecution,
    /// Inspection failed before execution.
    FailedBeforeExecution,
}
