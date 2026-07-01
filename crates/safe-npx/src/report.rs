//! Human and JSON M1 report rendering.

use crate::inspect_json_schema::{
    build_inspect_json_report, build_m2_execution_refusal_json_report,
};
use crate::m2_report::{
    closure_decision_for_m2_reasons, closure_decision_name, exit_code_for_closure_decision,
    format_m2_reasons, required_next_action_for_m2_reasons, required_next_action_name,
};
use crate::report_inspect::{
    build_inspect_model, render_model_facts, render_model_intent, render_model_summary,
};
use crate::{
    evaluate_m1_policy, extract_for_inspect, inspect_raw_spec_with_probe,
    serialize_redacted_string, ArtifactIdentity, Cli, ClosureCommandIdentity, ClosureDecision,
    CommandIntent, CountingProbe, Decision, InspectModel, M1Reason, M2Reason, NpmMetadataClient,
    PackageSpecParse, PolicyDecision, PolicyNextAction, RegistryEvidence, RegistryTransport,
    ReqwestRegistryTransport, ReqwestTarballTransport, ResolvedPackage, RootArtifactResolver,
    SourceContext, StaticExtractionEvidence, TarballDownloader, TarballTransport,
};
use serde::Serialize;

/// M4 exit code used for successful inspection or execution.
pub const M4_SUCCESS_EXIT_CODE: i32 = 0;
/// M4 exit code used when non-interactive policy needs a human question.
pub const M4_ASK_REQUIRED_EXIT_CODE: i32 = 10;
/// M4 exit code used when policy denies the command.
pub const M4_DENIED_EXIT_CODE: i32 = 11;
/// M4 exit code used when the requested command shape is unsupported.
pub const M4_UNSUPPORTED_EXIT_CODE: i32 = 12;
/// M4 exit code used when evidence cannot be collected reliably.
pub const M4_INSPECTION_ERROR_EXIT_CODE: i32 = 13;
/// M4 exit code used when execution closure proof fails before package code can run.
pub const M4_EXECUTION_REFUSED_EXIT_CODE: i32 = 14;
/// M4 exit code reserved for delegated execution failure.
pub const M4_DELEGATED_EXECUTION_FAILED_EXIT_CODE: i32 = 15;
/// Compatibility alias for M2 execution-refusal paths.
pub const M2_EXECUTION_REFUSED_EXIT_CODE: i32 = M4_EXECUTION_REFUSED_EXIT_CODE;
/// Compatibility alias for M2 unsupported-command paths.
pub const M2_UNSUPPORTED_EXIT_CODE: i32 = M4_UNSUPPORTED_EXIT_CODE;

/// CLI output and process status.
#[derive(Debug, PartialEq, Eq)]
pub struct CliRunOutput {
    /// Text that should be written to stdout.
    pub stdout: String,
    /// Process exit code that should be returned by the binary.
    pub exit_code: i32,
}

/// Scaffold inspection report emitted for humans and agents.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    /// Package spec requested by the caller.
    #[serde(serialize_with = "serialize_redacted_string")]
    pub package_spec: String,
    /// Parsed command intent before registry or artifact access.
    pub intent: CommandIntent,
    /// Current recommendation from the policy gate.
    pub recommendation: Decision,
    /// Implementation status for the scaffold.
    pub status: &'static str,
    /// Human-readable note explaining the current boundary.
    pub note: &'static str,
    /// Shared inspect model consumed by human and JSON renderers.
    pub inspect: InspectModel,
    /// M1 resolver and artifact evidence.
    pub m1: M1Evidence,
}

/// M1 report evidence emitted before any package execution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum M1Evidence {
    /// Input was refused before registry or tarball access.
    NoDownload {
        /// Stable reason for the refusal.
        reason: M1Reason,
        /// Whether any registry or tarball bytes were downloaded.
        downloaded: bool,
    },
    /// Root artifact was downloaded and verified against registry integrity.
    Verified {
        /// Exact package metadata resolved from npm.
        resolved_package: ResolvedPackage,
        /// Integrity verification status.
        integrity_status: &'static str,
        /// Verified digest identity for the exact root artifact bytes.
        artifact_identity: ArtifactIdentity,
        /// Registry metadata evidence tied to the resolved exact version.
        registry_evidence: RegistryEvidence,
        /// Static extraction evidence collected from verified bytes.
        #[serde(skip_serializing_if = "Option::is_none")]
        static_extraction: Option<StaticExtractionEvidence>,
    },
    /// Registry, download, or integrity work failed before execution.
    Failed {
        /// Stable M1 reason for the failure.
        reason: M1Reason,
        /// Whether root artifact bytes were downloaded before the failure.
        downloaded: bool,
        /// Short deterministic detail for humans and tests.
        #[serde(serialize_with = "serialize_redacted_string")]
        detail: String,
    },
}

/// Report emitted when M2 refuses execution after inspection succeeds.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct M2ExecutionRefusalReport {
    /// Command identity requested by the caller.
    pub command: ClosureCommandIdentity,
    /// Stable execution decision.
    pub decision: ClosureDecision,
    /// Stable machine-readable refusal reasons.
    pub reasons: Vec<M2Reason>,
    /// Next action available to a human or agent.
    pub required_next_action: RequiredNextAction,
    /// Execution evidence; null because refused paths never run package code.
    pub execution: Option<ExecutionReport>,
    /// Deterministic process exit code for this M2 stop.
    pub exit_code: i32,
}

/// Agent-readable next action vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequiredNextAction {
    /// No action is required.
    None,
    /// Ask a human before proceeding.
    AskUser,
    /// Retry with a narrower exact command shape.
    RetryNarrowerCommand,
    /// Inspect only; execution is not available for this closure.
    InspectOnly,
    /// Use a future explicit override path.
    ExplicitOverride,
    /// The requested command shape is unsupported.
    Unsupported,
}

/// Execution details populated only when package code actually runs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExecutionReport {
    /// Executed program path or identifier.
    pub program: String,
    /// Discrete argv passed to the program.
    pub argv: Vec<String>,
    /// Process exit code returned by execution.
    pub exit_code: Option<i32>,
}

/// Build the current scaffold report from parsed CLI arguments.
pub fn build_report(cli: &Cli) -> Report {
    let resolver = RootArtifactResolver::new(
        NpmMetadataClient::public(ReqwestRegistryTransport),
        TarballDownloader::new(ReqwestTarballTransport),
    );

    build_report_with_resolver(cli, &resolver)
}

/// Build a report from parsed CLI arguments and a supplied resolver.
pub fn build_report_with_resolver<M: RegistryTransport, D: TarballTransport>(
    cli: &Cli,
    resolver: &RootArtifactResolver<M, D>,
) -> Report {
    let raw_package_spec = cli.raw_package_spec();
    let mut probe = CountingProbe::default();
    let intent = inspect_raw_spec_with_probe(&raw_package_spec, cli.forwarded_args(), &mut probe);
    let (recommendation, m1) = match &intent.package_spec {
        PackageSpecParse::Supported(package_spec) => match resolver.resolve(package_spec) {
            Ok(verified) => {
                if cli.is_inspect_action() {
                    match extract_for_inspect(&verified.artifact_bytes, &verified.artifact_identity)
                    {
                        Ok(static_extraction) => (
                            cli.decision.clone(),
                            M1Evidence::Verified {
                                resolved_package: verified.resolved_package,
                                integrity_status: "verified",
                                artifact_identity: verified.artifact_identity,
                                registry_evidence: verified.registry_evidence,
                                static_extraction: Some(static_extraction),
                            },
                        ),
                        Err(error) => (
                            Decision::Ask,
                            M1Evidence::Failed {
                                reason: M1Reason::RegistryError,
                                downloaded: true,
                                detail: format!("static extraction failed: {}", error.detail),
                            },
                        ),
                    }
                } else {
                    (
                        cli.decision.clone(),
                        M1Evidence::Verified {
                            resolved_package: verified.resolved_package,
                            integrity_status: "verified",
                            artifact_identity: verified.artifact_identity,
                            registry_evidence: verified.registry_evidence,
                            static_extraction: None,
                        },
                    )
                }
            }
            Err(error) => {
                let downloaded = error.reason == M1Reason::IntegrityMismatch;
                (
                    error.decision.unwrap_or(Decision::Ask),
                    M1Evidence::Failed {
                        reason: error.reason,
                        downloaded,
                        detail: error.detail,
                    },
                )
            }
        },
        PackageSpecParse::Unsupported(unsupported) => (
            Decision::Deny,
            M1Evidence::NoDownload {
                reason: unsupported.reason.clone(),
                downloaded: unsupported.downloaded,
            },
        ),
        PackageSpecParse::Malformed(malformed) => (
            Decision::Deny,
            M1Evidence::NoDownload {
                reason: malformed.reason.clone(),
                downloaded: malformed.downloaded,
            },
        ),
    };

    let inspect = build_inspect_model(&intent, &recommendation, &m1, &cli.source_context);

    Report {
        package_spec: raw_package_spec,
        intent,
        recommendation,
        status: if cli.is_inspect_action() {
            "m3_inspect"
        } else {
            "m1_evidence"
        },
        note: if cli.is_inspect_action() {
            "M3 inspect resolves, verifies, statically extracts package metadata, and stops before package code can run."
        } else {
            "M1 resolves and verifies the root artifact only; dependency graphs, lifecycle scripts, maintainer reputation, and policy scoring are not implemented yet."
        },
        inspect,
        m1,
    }
}

/// Render the report in the requested output format.
pub fn render_report(cli: &Cli, report: &Report) -> anyhow::Result<String> {
    if cli.json {
        return Ok(serde_json::to_string_pretty(&build_inspect_json_report(
            report,
        ))?);
    }

    let policy = evaluate_m1_policy(&report.recommendation, &report.m1);
    let non_interactive_ask_required = policy_requires_interaction(&policy)
        && source_context_is_non_interactive(
            &report.inspect.authority_context.redacted.source_context,
        );

    Ok(format!(
        "Package: {}\nStatus: {}\nRecommendation: {:?}\n\n[Command]\n{}[Facts]\n{}[Decision]\n{}[Safety boundary]\n{}",
        crate::redact_report_value(&report.package_spec),
        report.status,
        report.inspect.decision.recommendation,
        render_model_intent(&report.inspect),
        render_model_facts(&report.inspect),
        render_model_summary(&report.inspect, non_interactive_ask_required),
        safety_boundary(report)
    ))
}

/// Render the human safety boundary for the report capability level.
fn safety_boundary(report: &Report) -> &'static str {
    if report.status == "m3_inspect" {
        return "safe-npx catches unsupported command shapes, integrity mismatches, lifecycle/dependency surfaces, and authority-context risk signals.\nRisk signals are not proof that a package is safe.\nThis Rust CLI does not execute package code in M1.\nM3 inspect does not verify dependency closure or execute package code.\n";
    }

    "safe-npx catches unsupported command shapes and root artifact integrity mismatches in M1.\nRisk signals are not proof that a package is safe.\nThis Rust CLI does not execute package code in M1.\nNext step: use inspect mode for static lifecycle/dependency evidence.\n"
}

/// Build a deterministic M2 refusal report for an unproven closure.
pub fn build_m2_execution_refusal_report(
    command: ClosureCommandIdentity,
    reasons: Vec<M2Reason>,
) -> M2ExecutionRefusalReport {
    let decision = closure_decision_for_m2_reasons(&reasons);
    let required_next_action = required_next_action_for_m2_reasons(&reasons);
    let exit_code = if reasons.contains(&M2Reason::NonInteractiveStop) {
        M4_ASK_REQUIRED_EXIT_CODE
    } else {
        exit_code_for_closure_decision(&decision)
    };

    M2ExecutionRefusalReport {
        command,
        decision: decision.clone(),
        reasons,
        required_next_action,
        execution: None,
        exit_code,
    }
}

/// Convert an interactive M2 ask into a non-interactive stop.
pub fn build_m2_non_interactive_stop_report(
    command: ClosureCommandIdentity,
) -> M2ExecutionRefusalReport {
    build_m2_execution_refusal_report(command, vec![M2Reason::NonInteractiveStop])
}

/// Render an M2 execution refusal report in the requested output format.
pub fn render_m2_execution_refusal_report(
    cli: &Cli,
    report: &M2ExecutionRefusalReport,
) -> anyhow::Result<String> {
    if cli.json {
        return Ok(serde_json::to_string_pretty(
            &build_m2_execution_refusal_json_report(report),
        )?);
    }

    Ok(format!(
        "Execution refused\nPackage: {}\nDecision: {}\nReasons: {}\nRequired next action: {}\nExit code: {}\nNo package code was executed.\n",
        crate::redact_report_value(&report.command.requested),
        closure_decision_name(&report.decision),
        format_m2_reasons(&report.reasons),
        required_next_action_name(&report.required_next_action),
        report.exit_code
    ))
}

/// Run the CLI and return the output that should be written to stdout.
pub fn run(cli: &Cli) -> anyhow::Result<String> {
    let report = build_report(cli);
    render_report(cli, &report)
}

/// Run the CLI and return stdout plus the process exit code.
pub fn run_with_exit_code(cli: &Cli) -> anyhow::Result<CliRunOutput> {
    if let Some(reason) = &cli.m2_refusal {
        let report = build_m2_execution_refusal_report(
            ClosureCommandIdentity {
                requested: cli.raw_package_spec(),
                forwarded_args: cli.forwarded_args(),
            },
            vec![M2Reason::from(reason.clone())],
        );
        let exit_code = report.exit_code;
        return Ok(CliRunOutput {
            stdout: render_m2_execution_refusal_report(cli, &report)?,
            exit_code,
        });
    }

    let report = build_report(cli);
    let exit_code = exit_code_for_report(&report);
    Ok(CliRunOutput {
        stdout: render_report(cli, &report)?,
        exit_code,
    })
}

/// Return the process exit code implied by the current report.
pub(crate) fn exit_code_for_report(report: &Report) -> i32 {
    let policy = evaluate_m1_policy(&report.recommendation, &report.m1);
    if policy_requires_interaction(&policy)
        && source_context_is_non_interactive(
            &report.inspect.authority_context.redacted.source_context,
        )
    {
        return M4_ASK_REQUIRED_EXIT_CODE;
    }

    exit_code_for_policy_decision(&policy.decision)
}

/// Return the process exit code for a canonical M4 policy decision.
pub(crate) fn exit_code_for_policy_decision(decision: &PolicyDecision) -> i32 {
    match decision {
        PolicyDecision::Allow | PolicyDecision::Ask => M4_SUCCESS_EXIT_CODE,
        PolicyDecision::Deny => M4_DENIED_EXIT_CODE,
        PolicyDecision::Unsupported => M4_UNSUPPORTED_EXIT_CODE,
        PolicyDecision::InspectionError => M4_INSPECTION_ERROR_EXIT_CODE,
        PolicyDecision::ExecutionRefused => M4_EXECUTION_REFUSED_EXIT_CODE,
    }
}

/// Return true when policy cannot proceed without a human answer.
pub(crate) fn policy_requires_interaction(policy: &crate::PolicyEvaluation) -> bool {
    matches!(policy.decision, PolicyDecision::Ask)
        && matches!(policy.required_next_action, PolicyNextAction::AskUser)
}

/// Return true when the declared source context cannot answer a terminal prompt.
pub(crate) fn source_context_is_non_interactive(source_context: &SourceContext) -> bool {
    matches!(
        source_context,
        SourceContext::AgentSkill | SourceContext::Ci
    )
}
