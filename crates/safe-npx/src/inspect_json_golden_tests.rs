//! Expanded deterministic golden tests for M3 inspect JSON.

use crate::inspect_golden_fixtures::{
    integrity_failure_report, missing_optional_metadata_report, redacted_authority_report,
    static_blockers_report,
};
use crate::{render_report, Cli, Report};
use serde_json::Value;
use std::path::{Path, PathBuf};

const INTEGRITY_FAILURE_GOLDEN: &str =
    include_str!("../fixtures/inspect-json-schema-v0-integrity-failure.json");
const STATIC_BLOCKERS_GOLDEN: &str =
    include_str!("../fixtures/inspect-json-schema-v0-static-blockers.json");
const REDACTED_AUTHORITY_GOLDEN: &str =
    include_str!("../fixtures/inspect-json-schema-v0-redacted-authority.json");
const MISSING_OPTIONAL_METADATA_GOLDEN: &str =
    include_str!("../fixtures/inspect-json-schema-v0-missing-optional-metadata.json");

/// Verifies expanded M3 inspect JSON fixtures stay byte-stable.
#[test]
fn expanded_inspect_json_golden_fixtures_are_stable() {
    let cli = Cli::parse_from(["safe-npx", "--json", "create-example@1.2.3"]);
    let integrity_failure = render_schema_golden(&cli, &integrity_failure_report());
    let static_blockers = render_schema_golden(&cli, &static_blockers_report());
    let redacted_authority = render_schema_golden(&cli, &redacted_authority_report());
    let missing_optional_metadata = render_schema_golden(&cli, &missing_optional_metadata_report());

    let cases = [
        (
            "inspect-json-schema-v0-integrity-failure.json",
            integrity_failure.as_str(),
        ),
        (
            "inspect-json-schema-v0-static-blockers.json",
            static_blockers.as_str(),
        ),
        (
            "inspect-json-schema-v0-redacted-authority.json",
            redacted_authority.as_str(),
        ),
        (
            "inspect-json-schema-v0-missing-optional-metadata.json",
            missing_optional_metadata.as_str(),
        ),
    ];

    maybe_print_schema_goldens(&cases);
    if maybe_update_schema_goldens(&cases) {
        return;
    }

    assert_eq!(integrity_failure, INTEGRITY_FAILURE_GOLDEN);
    assert_eq!(static_blockers, STATIC_BLOCKERS_GOLDEN);
    assert_eq!(redacted_authority, REDACTED_AUTHORITY_GOLDEN);
    assert_eq!(missing_optional_metadata, MISSING_OPTIONAL_METADATA_GOLDEN);
}

/// Verifies redacted authority output does not expose fixture secrets or host paths.
#[test]
fn redacted_authority_golden_hides_secret_and_host_inputs() {
    let cli = Cli::parse_from(["safe-npx", "--json", "create-example@1.2.3"]);
    let output = render_schema_golden(&cli, &redacted_authority_report());

    for forbidden in [
        "sekret-token",
        "sekret-auth",
        "/home/example",
        "_authToken=sekret-auth",
    ] {
        assert!(
            !output.contains(forbidden),
            "redacted authority output leaked {forbidden}"
        );
    }
}

/// Verifies expanded fixture outputs keep future hosted evidence fields null.
#[test]
fn expanded_json_golden_fixtures_keep_reserved_fields_null() {
    for (name, fixture) in [
        (
            "inspect-json-schema-v0-integrity-failure.json",
            INTEGRITY_FAILURE_GOLDEN,
        ),
        (
            "inspect-json-schema-v0-static-blockers.json",
            STATIC_BLOCKERS_GOLDEN,
        ),
        (
            "inspect-json-schema-v0-redacted-authority.json",
            REDACTED_AUTHORITY_GOLDEN,
        ),
        (
            "inspect-json-schema-v0-missing-optional-metadata.json",
            MISSING_OPTIONAL_METADATA_GOLDEN,
        ),
    ] {
        let value: Value = serde_json::from_str(fixture).expect("fixture should parse");
        for field in ["external_evidence", "attestations", "release_diff"] {
            assert_eq!(
                value.get(field),
                Some(&Value::Null),
                "{name} must keep {field} present and null"
            );
        }
    }
}

/// Verifies schema documentation records compatibility rules enforced by tests.
#[test]
fn schema_docs_record_enum_and_additive_field_compatibility_rules() {
    let schema_doc = schema_doc();
    for required in [
        "Additive fields are allowed within `0.x`.",
        "Enum additions require a schema bump.",
        "Enum semantic changes require a migration note.",
        "During the `0.1` transition",
    ] {
        assert!(
            schema_doc.contains(required),
            "missing doc rule: {required}"
        );
    }
}

/// Renders a report through the public JSON path with a trailing newline.
fn render_schema_golden(cli: &Cli, report: &Report) -> String {
    format!(
        "{}\n",
        render_report(cli, report).expect("schema should render")
    )
}

/// Prints regenerated fixture contents when explicitly requested.
fn maybe_print_schema_goldens(cases: &[(&str, &str)]) {
    if std::env::var_os("SAFE_NPX_PRINT_SCHEMA_GOLDENS").is_none() {
        return;
    }

    for (name, value) in cases {
        eprintln!("--- {name} ---\n{value}");
    }
}

/// Updates fixture files when explicitly requested by a developer.
fn maybe_update_schema_goldens(cases: &[(&str, &str)]) -> bool {
    if std::env::var_os("SAFE_NPX_UPDATE_SCHEMA_GOLDENS").is_none() {
        return false;
    }

    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    for (name, value) in cases {
        std::fs::write(fixture_root.join(name), value).expect("fixture should update");
    }
    true
}

/// Reads the schema doc from Cargo or Bazel test locations.
fn schema_doc() -> String {
    let relative_doc = Path::new("docs/inspect-json-schema-v0.md");
    let mut candidates = vec![PathBuf::from(relative_doc)];
    if let Ok(cwd) = std::env::current_dir() {
        candidates.extend(cwd.ancestors().map(|ancestor| ancestor.join(relative_doc)));
    }
    if let Some(manifest_dir) = std::env::var_os("CARGO_MANIFEST_DIR") {
        candidates.extend(
            PathBuf::from(manifest_dir)
                .ancestors()
                .map(|ancestor| ancestor.join(relative_doc)),
        );
    }
    if let Some(srcdir) = std::env::var_os("TEST_SRCDIR") {
        let srcdir = PathBuf::from(srcdir);
        candidates.push(srcdir.join("_main").join(relative_doc));
        candidates.push(srcdir.join("__main__").join(relative_doc));
        candidates.push(srcdir.join(relative_doc));
    }
    if let (Some(srcdir), Some(workspace)) = (
        std::env::var_os("TEST_SRCDIR"),
        std::env::var_os("TEST_WORKSPACE"),
    ) {
        candidates.push(PathBuf::from(srcdir).join(workspace).join(relative_doc));
    }

    for candidate in candidates {
        if let Ok(contents) = std::fs::read_to_string(&candidate) {
            return contents;
        }
    }

    panic!("could not locate docs/inspect-json-schema-v0.md");
}
