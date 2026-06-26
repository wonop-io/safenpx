//! Deterministic golden tests for M3 inspect human reports.

use crate::inspect_golden_fixtures::{
    integrity_failure_report, missing_optional_metadata_report, normal_report,
    redacted_authority_report, static_blockers_report, unsupported_report,
};
use crate::{render_report, Cli, Report};
use std::path::Path;

const NORMAL_GOLDEN: &str = include_str!("../fixtures/inspect-human-normal.txt");
const STATIC_BLOCKERS_GOLDEN: &str = include_str!("../fixtures/inspect-human-static-blockers.txt");
const UNSUPPORTED_GOLDEN: &str = include_str!("../fixtures/inspect-human-unsupported.txt");
const MISSING_OPTIONAL_METADATA_GOLDEN: &str =
    include_str!("../fixtures/inspect-human-missing-optional-metadata.txt");
const REDACTED_AUTHORITY_GOLDEN: &str =
    include_str!("../fixtures/inspect-human-redacted-authority.txt");
const INTEGRITY_FAILURE_GOLDEN: &str =
    include_str!("../fixtures/inspect-human-integrity-failure.txt");

/// Verifies human inspect reports stay byte-stable for the M3 evidence matrix.
#[test]
fn inspect_human_golden_fixtures_are_stable() {
    let cli = Cli::parse_from(["safe-npx", "create-example@1.2.3"]);
    let normal = render_human_golden(&cli, &normal_report());
    let static_blockers = render_human_golden(&cli, &static_blockers_report());
    let unsupported = render_human_golden(&cli, &unsupported_report());
    let missing_optional_metadata = render_human_golden(&cli, &missing_optional_metadata_report());
    let redacted_authority = render_human_golden(&cli, &redacted_authority_report());
    let integrity_failure = render_human_golden(&cli, &integrity_failure_report());

    let cases = [
        ("inspect-human-normal.txt", normal.as_str()),
        (
            "inspect-human-static-blockers.txt",
            static_blockers.as_str(),
        ),
        ("inspect-human-unsupported.txt", unsupported.as_str()),
        (
            "inspect-human-missing-optional-metadata.txt",
            missing_optional_metadata.as_str(),
        ),
        (
            "inspect-human-redacted-authority.txt",
            redacted_authority.as_str(),
        ),
        (
            "inspect-human-integrity-failure.txt",
            integrity_failure.as_str(),
        ),
    ];

    maybe_print_human_goldens(&cases);
    if maybe_update_human_goldens(&cases) {
        return;
    }

    assert_eq!(normal, NORMAL_GOLDEN);
    assert_eq!(static_blockers, STATIC_BLOCKERS_GOLDEN);
    assert_eq!(unsupported, UNSUPPORTED_GOLDEN);
    assert_eq!(missing_optional_metadata, MISSING_OPTIONAL_METADATA_GOLDEN);
    assert_eq!(redacted_authority, REDACTED_AUTHORITY_GOLDEN);
    assert_eq!(integrity_failure, INTEGRITY_FAILURE_GOLDEN);
}

/// Verifies human authority redaction does not leak fixture secrets or host paths.
#[test]
fn redacted_authority_human_golden_hides_secret_and_host_inputs() {
    let cli = Cli::parse_from(["safe-npx", "create-example@1.2.3"]);
    let output = format!(
        "{}\n{}",
        render_human_golden(&cli, &redacted_authority_report()),
        render_human_golden(&cli, &normal_report())
    );

    for forbidden in [
        "sekret-token",
        "sekret-auth",
        "metadata-secret",
        "/home/example",
        "_authToken=sekret-auth",
    ] {
        assert!(
            !output.contains(forbidden),
            "human authority output leaked {forbidden}"
        );
    }
}

/// Renders a report through the public human report path with a trailing newline.
fn render_human_golden(cli: &Cli, report: &Report) -> String {
    format!(
        "{}\n",
        render_report(cli, report).expect("human report should render")
    )
}

/// Prints regenerated fixture contents when explicitly requested.
fn maybe_print_human_goldens(cases: &[(&str, &str)]) {
    if std::env::var_os("SAFE_NPX_PRINT_HUMAN_GOLDENS").is_none() {
        return;
    }

    for (name, value) in cases {
        eprintln!("--- {name} ---\n{value}");
    }
}

/// Updates fixture files when explicitly requested by a developer.
fn maybe_update_human_goldens(cases: &[(&str, &str)]) -> bool {
    if std::env::var_os("SAFE_NPX_UPDATE_HUMAN_GOLDENS").is_none() {
        return false;
    }

    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    for (name, value) in cases {
        std::fs::write(fixture_root.join(name), value).expect("fixture should update");
    }
    true
}
