//! M3 inspect decision receipt shape.

use crate::{
    redact_report_value, redact_report_values, ArtifactIdentity, InspectExecutionStateKind,
    InspectJsonDecision, InspectJsonNextAction, PackageSpecParse, Report,
};
use serde::Serialize;

/// Receipt shape for local or shareable inspect records.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InspectDecisionReceipt {
    /// Receipt schema version.
    pub receipt_version: &'static str,
    /// Inspect schema version that produced the receipt.
    pub schema_version: &'static str,
    /// Provisional policy version; semantics are defined after M3.
    pub policy_version: &'static str,
    /// Receipt timestamp; null in normal M3 inspect JSON.
    pub issued_at: Option<String>,
    /// Explicitly states that M3 receipts are not execution approvals.
    pub cache_status: &'static str,
    /// Artifact identity covered by the receipt, when known.
    pub artifact: Option<ReceiptArtifactIdentity>,
    /// Command identity covered by the receipt.
    pub command: ReceiptCommandIdentity,
    /// Evidence and decision summary.
    pub evidence_summary: ReceiptEvidenceSummary,
    /// Redaction and identity metadata.
    pub redaction: ReceiptRedactionMetadata,
}

/// Canonical artifact identity fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReceiptArtifactIdentity {
    /// Package name from verified artifact identity.
    pub name: String,
    /// Package version from verified artifact identity.
    pub version: String,
    /// Integrity metadata used for verification.
    pub integrity: String,
    /// Digest algorithm for the verified root artifact.
    pub digest_algorithm: String,
    /// Digest for the verified root artifact.
    pub digest: String,
}

/// Command identity separated from redacted display values.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReceiptCommandIdentity {
    /// Canonical command identity key from authority metadata.
    pub identity_key: String,
    /// Parsed package-spec state.
    pub package_state: &'static str,
    /// Package name when an exact package was parsed.
    pub package_name: Option<String>,
    /// Package version when an exact package was parsed.
    pub package_version: Option<String>,
    /// Number of forwarded args.
    pub forwarded_args_count: usize,
    /// Redacted command display for humans.
    pub requested_display: String,
    /// Redacted forwarded args display for humans.
    pub forwarded_args_display: Vec<String>,
}

/// Receipt evidence summary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReceiptEvidenceSummary {
    /// Public JSON decision.
    pub decision: InspectJsonDecision,
    /// Public JSON required next action.
    pub required_next_action: InspectJsonNextAction,
    /// Stable decision reasons.
    pub reasons: Vec<String>,
    /// Process exit code implied by the report.
    pub exit_code: i32,
    /// Execution state captured before package code can run.
    pub execution_state: InspectExecutionStateKind,
    /// Whether package code executed.
    pub package_code_executed: bool,
    /// Count of report-only heuristics.
    pub heuristics_count: usize,
}

/// Redaction metadata carried by the receipt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReceiptRedactionMetadata {
    /// Display values are redacted before serialization.
    pub display_redaction: &'static str,
    /// Authority identity metadata version.
    pub authority_identity_status: &'static str,
    /// Canonical command identity key.
    pub command_intent_key: String,
    /// Canonical cwd trust class, not a raw path.
    pub cwd_trust_class: String,
    /// Canonical registry key without credentials.
    pub registry_key: Option<String>,
    /// Boundary reminder for later cache/receipt semantics.
    pub boundary: &'static str,
}

/// Build a non-authoritative M3 inspect receipt.
pub fn build_inspect_decision_receipt(
    report: &Report,
    decision: InspectJsonDecision,
    required_next_action: InspectJsonNextAction,
    exit_code: i32,
) -> InspectDecisionReceipt {
    let authority = &report.inspect.authority_context.redacted;

    InspectDecisionReceipt {
        receipt_version: "m3-inspect-receipt-v0",
        schema_version: "0.1",
        policy_version: "m3-inspect-policy-v0",
        issued_at: None,
        cache_status: "not_an_approval_cache",
        artifact: report
            .inspect
            .facts
            .artifact
            .as_ref()
            .map(receipt_artifact_identity),
        command: receipt_command_identity(report),
        evidence_summary: ReceiptEvidenceSummary {
            decision,
            required_next_action,
            reasons: report.inspect.decision.reasons.clone(),
            exit_code,
            execution_state: report.inspect.execution_state.state.clone(),
            package_code_executed: report.inspect.execution_state.package_code_executed,
            heuristics_count: report.inspect.heuristics.len(),
        },
        redaction: ReceiptRedactionMetadata {
            display_redaction: "redacted_report_v0",
            authority_identity_status: authority.identity.status,
            command_intent_key: authority.identity.command_intent_key.clone(),
            cwd_trust_class: authority.identity.cwd_trust_class.clone(),
            registry_key: authority.identity.registry_key.clone(),
            boundary:
                "receipt is inspect evidence only; M3 does not define approval-cache semantics",
        },
    }
}

/// Convert verified artifact identity into receipt identity.
fn receipt_artifact_identity(artifact: &ArtifactIdentity) -> ReceiptArtifactIdentity {
    ReceiptArtifactIdentity {
        name: artifact.name.clone(),
        version: artifact.version.clone(),
        integrity: artifact.integrity.clone(),
        digest_algorithm: artifact.digest_algorithm.clone(),
        digest: artifact.digest.clone(),
    }
}

/// Build command identity with canonical and display fields separated.
fn receipt_command_identity(report: &Report) -> ReceiptCommandIdentity {
    let (package_state, package_name, package_version) = match &report.intent.package_spec {
        PackageSpecParse::Supported(package) => (
            "supported",
            Some(package.name.clone()),
            Some(package.version.clone()),
        ),
        PackageSpecParse::Unsupported(_) => ("unsupported", None, None),
        PackageSpecParse::Malformed(_) => ("malformed", None, None),
    };

    ReceiptCommandIdentity {
        identity_key: report
            .inspect
            .authority_context
            .redacted
            .identity
            .command_intent_key
            .clone(),
        package_state,
        package_name,
        package_version,
        forwarded_args_count: report.intent.forwarded_args.len(),
        requested_display: redact_report_value(&report.intent.requested),
        forwarded_args_display: redact_report_values(&report.intent.forwarded_args),
    }
}
