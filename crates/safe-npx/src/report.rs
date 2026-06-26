//! Human and JSON M1 report rendering.

use crate::m2_report::{
    closure_decision_for_m2_reasons, closure_decision_name, exit_code_for_closure_decision,
    format_m2_reasons, required_next_action_for_m2_reasons, required_next_action_name,
};
use crate::{
    extract_for_inspect, inspect_raw_spec_with_probe, render_static_extraction, ArtifactIdentity,
    Cli, ClosureCommandIdentity, ClosureDecision, CommandIntent, CountingProbe, Decision, M1Reason,
    M2Reason, NpmMetadataClient, PackageSpecParse, RegistryTransport, ReqwestRegistryTransport,
    ReqwestTarballTransport, ResolvedPackage, RootArtifactResolver, StaticExtractionEvidence,
    TarballDownloader, TarballTransport, UnsupportedSpecCategory,
};
use serde::Serialize;

/// M2 exit code used when execution closure proof fails before package code can run.
pub const M2_EXECUTION_REFUSED_EXIT_CODE: i32 = 5;
/// M2 exit code used when the requested execution shape is unsupported.
pub const M2_UNSUPPORTED_EXIT_CODE: i32 = 2;

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
    let decision = closure_decision_for_m2_reasons(&reasons);
    let required_next_action = required_next_action_for_m2_reasons(&reasons);

    M2ExecutionRefusalReport {
        command,
        decision: decision.clone(),
        reasons,
        required_next_action,
        execution: None,
        exit_code: exit_code_for_closure_decision(&decision),
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

    Ok(CliRunOutput {
        stdout: run(cli)?,
        exit_code: 0,
    })
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
            static_extraction,
        } => format!(
            "M1 evidence: verified\nResolved: {}@{}\nRegistry: {}\nTarball: {}\nIntegrity: {}\nIntegrity metadata: {}\nDigest: {}:{}\n{}",
            resolved_package.name,
            resolved_package.version,
            resolved_package.registry.url,
            resolved_package.tarball_url,
            integrity_status,
            resolved_package.integrity,
            artifact_identity.digest_algorithm,
            artifact_identity.digest,
            render_static_extraction(static_extraction.as_ref())
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

/// Format forwarded CLI arguments for human output.
fn format_forwarded_args(args: &[String]) -> String {
    if args.is_empty() {
        return "[]".to_string();
    }

    args.join(" ")
}

/// Return the stable serialized name for an M1 reason.
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

/// Return the stable serialized name for an unsupported spec category.
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
