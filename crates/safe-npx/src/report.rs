//! Human and JSON M1 report rendering.

use crate::{
    inspect_raw_spec_with_probe, ArtifactIdentity, Cli, CommandIntent, CountingProbe, Decision,
    M1Reason, NpmMetadataClient, PackageSpecParse, RegistryTransport, ReqwestRegistryTransport,
    ReqwestTarballTransport, ResolvedPackage, RootArtifactResolver, TarballDownloader,
    TarballTransport, UnsupportedSpecCategory,
};
use serde::Serialize;

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
