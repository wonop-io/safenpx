//! Tests for the M3 inspect decision receipt shape.

use crate::inspect_golden_fixtures::normal_report;
use crate::{build_inspect_json_report, build_m2_execution_refusal_json_report};
use crate::{build_m2_execution_refusal_report, ClosureCommandIdentity, M2Reason};
use serde_json::Value;

/// Verifies the M3 receipt shape is present but not an approval cache.
#[test]
fn inspect_json_decision_receipt_shape_is_non_authoritative() {
    let value =
        serde_json::to_value(build_inspect_json_report(&normal_report())).expect("serialize");
    let receipt = &value["decision_receipt"];

    assert_eq!(receipt["receipt_version"], "m3-inspect-receipt-v0");
    assert_eq!(receipt["schema_version"], "0.1");
    assert_eq!(receipt["policy_version"], "m3-inspect-policy-v0");
    assert_eq!(receipt["issued_at"], Value::Null);
    assert_eq!(receipt["cache_status"], "not_an_approval_cache");
    assert_eq!(receipt["artifact"]["digest_algorithm"], "sha512");
    assert_eq!(receipt["artifact"]["digest"], "digest-example");
    assert_eq!(receipt["command"]["package_state"], "supported");
    assert_eq!(receipt["command"]["package_name"], "create-example");
    assert_eq!(receipt["command"]["package_version"], "1.2.3");
    assert!(receipt["command"]["identity_key"]
        .as_str()
        .expect("identity key should be a string")
        .starts_with("sha256:"));
    assert_eq!(receipt["evidence_summary"]["decision"], "ask");
    assert_eq!(
        receipt["evidence_summary"]["required_next_action"],
        "ask_user"
    );
    assert_eq!(
        receipt["redaction"]["authority_identity_status"],
        "canonical_redacted_identity_v0"
    );
    assert_eq!(
        receipt["redaction"]["display_redaction"],
        "redacted_report_v0"
    );
    assert!(receipt["redaction"]["boundary"]
        .as_str()
        .expect("boundary should be a string")
        .contains("M3 does not define approval-cache semantics"));
}

/// Verifies execution-refused reports do not mint inspect receipts in M3.
#[test]
fn execution_refusal_json_has_null_decision_receipt() {
    let report = build_m2_execution_refusal_report(
        ClosureCommandIdentity {
            requested: "create-example@1.2.3".to_string(),
            forwarded_args: Vec::new(),
        },
        vec![M2Reason::UnsupportedClosure],
    );
    let value = build_m2_execution_refusal_json_report(&report);

    assert_eq!(value.get("decision_receipt"), Some(&Value::Null));
}
