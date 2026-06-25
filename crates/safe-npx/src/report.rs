//! Human and JSON M1 report rendering.

use crate::{
    inspect_raw_spec_with_probe, ArtifactIdentity, Cli, ClosureCommandIdentity, ClosureDecision,
    CommandIntent, CountingProbe, Decision, M1Reason, M2Reason, NpmMetadataClient,
    PackageSpecParse, RegistryTransport, ReqwestRegistryTransport, ReqwestTarballTransport,
    ResolvedPackage, RootArtifactResolver, TarballDownloader, TarballTransport,
    UnsupportedSpecCategory,
};
use serde::Serialize;

/// M2 exit code used when execution closure proof fails before package code can run.
pub const M2_EXECUTION_REFUSED_EXIT_CODE: i32 = 5;

/// Scaffold inspection report emitted for humans and agents.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    /// Package spec requested by the caller.
    pub package_spec: String,
    /// Parsed command intent before registry or artifact access.
    pub intent: CommandIntent,
    /// Current recommendation from the policy gate.
    pub recommendation: Decision,
    /// Implementation status for the scaffold.
    pub status: &'static str,
    /// Human-readable note explaining the current boundary.
    pub note: &'static str,
    /// M1 resolver and artifact evidence.
    pub m1: M1Evidence,
}

/// M1 report evidence emitted before any package execution.
#[derive(Debug, PartialEq, Eq, Serialize)]
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
    },
    /// Registry, download, or integrity work failed before execution.
    Failed {
        /// Stable M1 reason for the failure.
        reason: M1Reason,
        /// Whether root artifact bytes were downloaded before the failure.
        downloaded: bool,
        /// Short deterministic detail for humans and tests.
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
#[derive(Debug, PartialEq, Eq, Serialize)]
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
            Ok(verified) => (
                cli.decision.clone(),
                M1Evidence::Verified {
                    resolved_package: verified.resolved_package,
                    integrity_status: "verified",
                    artifact_identity: verified.artifact_identity,
                },
            ),
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

    Report {
        package_spec: raw_package_spec,
        intent,
        recommendation,
        status: "m1_evidence",
        note: "M1 resolves and verifies the root artifact only; dependency graphs, lifecycle scripts, maintainer reputation, and policy scoring are not implemented yet.",
        m1,
    }
}

/// Render the report in the requested output format.
pub fn render_report(cli: &Cli, report: &Report) -> anyhow::Result<String> {
    if cli.json {
        return Ok(serde_json::to_string_pretty(report)?);
    }

    let intent_line = render_intent(report);
    let evidence_line = render_evidence(&report.m1);
    Ok(format!(
        "Package: {}\nStatus: {}\nRecommendation: {:?}\n{}{}\nThis Rust CLI does not execute package code in M1.\nNext step: expand evidence beyond the root artifact.\n",
        report.package_spec, report.status, report.recommendation, intent_line, evidence_line
    ))
}

/// Build a deterministic M2 refusal report for an unproven closure.
pub fn build_m2_execution_refusal_report(
    command: ClosureCommandIdentity,
    reasons: Vec<M2Reason>,
) -> M2ExecutionRefusalReport {
    let required_next_action = required_next_action_for_m2_reasons(&reasons);

    M2ExecutionRefusalReport {
        command,
        decision: ClosureDecision::ExecutionRefused,
        reasons,
        required_next_action,
        execution: None,
        exit_code: M2_EXECUTION_REFUSED_EXIT_CODE,
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
        return Ok(serde_json::to_string_pretty(report)?);
    }

    Ok(format!(
        "Execution refused\nPackage: {}\nDecision: {}\nReasons: {}\nRequired next action: {}\nExit code: {}\nNo package code was executed.\n",
        report.command.requested,
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

/// Render command intent for terminal output.
fn render_intent(report: &Report) -> String {
    match &report.intent.package_spec {
        PackageSpecParse::Supported(package_spec) => format!(
            "Parsed: {}@{}\nForwarded args: {}\n",
            package_spec.name,
            package_spec.version,
            format_forwarded_args(&report.intent.forwarded_args)
        ),
        PackageSpecParse::Unsupported(unsupported) => format!(
            "Rejected: {}\nReason: {}\nCategory: {}\nDownloaded: {}\n",
            report.intent.requested,
            reason_name(&unsupported.reason),
            unsupported_category_name(&unsupported.category),
            unsupported.downloaded
        ),
        PackageSpecParse::Malformed(malformed) => format!(
            "Rejected: {}\nReason: {}\nDownloaded: {}\n",
            report.intent.requested,
            reason_name(&malformed.reason),
            malformed.downloaded
        ),
    }
}

/// Render M1 evidence for terminal output.
fn render_evidence(m1: &M1Evidence) -> String {
    match m1 {
        M1Evidence::NoDownload { reason, downloaded } => format!(
            "M1 evidence: no_download\nReason: {}\nDownloaded: {}\n",
            reason_name(reason),
            downloaded
        ),
        M1Evidence::Verified {
            resolved_package,
            integrity_status,
            artifact_identity,
        } => format!(
            "M1 evidence: verified\nResolved: {}@{}\nRegistry: {}\nTarball: {}\nIntegrity: {}\nIntegrity metadata: {}\nDigest: {}:{}\n",
            resolved_package.name,
            resolved_package.version,
            resolved_package.registry.url,
            resolved_package.tarball_url,
            integrity_status,
            resolved_package.integrity,
            artifact_identity.digest_algorithm,
            artifact_identity.digest
        ),
        M1Evidence::Failed {
            reason,
            downloaded,
            detail,
        } => format!(
            "M1 evidence: failed\nReason: {}\nDownloaded: {}\nDetail: {}\n",
            reason_name(reason),
            downloaded,
            detail
        ),
    }
}

/// Format forwarded args for terminal output.
fn format_forwarded_args(args: &[String]) -> String {
    if args.is_empty() {
        return "[]".to_string();
    }

    args.join(" ")
}

/// Return the stable serialized reason name for human output.
fn reason_name(reason: &M1Reason) -> &'static str {
    match reason {
        M1Reason::UnsupportedSpec => "unsupported_spec",
        M1Reason::MalformedSpec => "malformed_spec",
        M1Reason::RegistryError => "registry_error",
        M1Reason::IntegrityMismatch => "integrity_mismatch",
        M1Reason::MissingPackage => "missing_package",
        M1Reason::MissingVersion => "missing_version",
    }
}

/// Return the stable serialized unsupported category name for human output.
fn unsupported_category_name(category: &UnsupportedSpecCategory) -> &'static str {
    match category {
        UnsupportedSpecCategory::UnversionedName => "unversioned_name",
        UnsupportedSpecCategory::VersionRange => "version_range",
        UnsupportedSpecCategory::GitUrl => "git_url",
        UnsupportedSpecCategory::LocalPath => "local_path",
        UnsupportedSpecCategory::TarballUrl => "tarball_url",
        UnsupportedSpecCategory::Alias => "alias",
        UnsupportedSpecCategory::MultipleSpecs => "multiple_specs",
        UnsupportedSpecCategory::NpmExecVariant => "npm_exec_variant",
        UnsupportedSpecCategory::Other => "other",
    }
}

/// Return the next action implied by M2 refusal reasons.
fn required_next_action_for_m2_reasons(reasons: &[M2Reason]) -> RequiredNextAction {
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

/// Format M2 reasons as stable comma-separated names for terminal output.
fn format_m2_reasons(reasons: &[M2Reason]) -> String {
    reasons
        .iter()
        .map(m2_reason_name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Return the stable serialized name for an M2 decision.
fn closure_decision_name(decision: &ClosureDecision) -> &'static str {
    match decision {
        ClosureDecision::Allow => "allow",
        ClosureDecision::Ask => "ask",
        ClosureDecision::Deny => "deny",
        ClosureDecision::Unsupported => "unsupported",
        ClosureDecision::InspectionError => "inspection_error",
        ClosureDecision::ExecutionRefused => "execution_refused",
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

/// Return the stable serialized name for a next action.
fn required_next_action_name(action: &RequiredNextAction) -> &'static str {
    match action {
        RequiredNextAction::None => "none",
        RequiredNextAction::AskUser => "ask_user",
        RequiredNextAction::RetryNarrowerCommand => "retry_narrower_command",
        RequiredNextAction::InspectOnly => "inspect_only",
        RequiredNextAction::ExplicitOverride => "explicit_override",
        RequiredNextAction::Unsupported => "unsupported",
    }
}
