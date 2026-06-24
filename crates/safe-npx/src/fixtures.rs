//! Fixture manifest support for M1 parser, resolver, and artifact tests.

/// M1 fixture manifest bundled with the crate.
pub const M1_FIXTURE_MANIFEST: &str = include_str!("../fixtures/m1-fixture-manifest.txt");

/// Parsed M1 fixture row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct M1Fixture {
    /// Stable fixture identifier.
    pub id: String,
    /// Fixture domain such as parser, registry, or artifact.
    pub kind: String,
    /// Raw package spec or command shape.
    pub raw_spec: String,
    /// Expected high-level result state.
    pub expected_state: String,
    /// Expected reason when relevant.
    pub expected_reason: Option<String>,
    /// Expected process exit code.
    pub expected_exit_code: i32,
    /// Forwarded args that should be preserved for supported specs.
    pub forwarded_args: Vec<String>,
    /// Sentinel expectation such as no_network or no_execution.
    pub sentinel: String,
}

/// Expected outcome seeded by a non-parser M1 fixture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct M1FixtureOutcome {
    /// Expected high-level result state.
    pub state: String,
    /// Expected reason when relevant.
    pub reason: Option<String>,
    /// Expected process exit code.
    pub exit_code: i32,
    /// Sentinel expectation such as no_execution.
    pub sentinel: String,
}

impl M1Fixture {
    /// Return true when this fixture is for parser behavior.
    pub fn is_parser_fixture(&self) -> bool {
        self.kind == "parser"
    }

    /// Return true when this fixture proves no network access is allowed.
    pub fn expects_no_network(&self) -> bool {
        self.sentinel == "no_network"
    }

    /// Return true when this fixture seeds artifact behavior.
    pub fn is_artifact_fixture(&self) -> bool {
        self.kind == "artifact"
    }

    /// Return true when this fixture seeds registry behavior.
    pub fn is_registry_fixture(&self) -> bool {
        self.kind == "registry"
    }

    /// Return the seeded outcome for registry and artifact failure fixtures.
    pub fn seeded_failure_outcome(&self) -> Option<M1FixtureOutcome> {
        if !self.is_registry_fixture() && !self.is_artifact_fixture() {
            return None;
        }

        Some(M1FixtureOutcome {
            state: self.expected_state.clone(),
            reason: self.expected_reason.clone(),
            exit_code: self.expected_exit_code,
            sentinel: self.sentinel.clone(),
        })
    }
}

/// Parse all M1 fixtures from the bundled manifest.
pub fn m1_fixtures() -> Vec<M1Fixture> {
    parse_m1_fixture_manifest(M1_FIXTURE_MANIFEST)
}

/// Parse an M1 fixture manifest.
pub fn parse_m1_fixture_manifest(manifest: &str) -> Vec<M1Fixture> {
    manifest
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(parse_fixture_line)
        .collect()
}

/// Return fixtures for one fixture kind.
pub fn fixtures_by_kind(kind: &str) -> Vec<M1Fixture> {
    m1_fixtures()
        .into_iter()
        .filter(|fixture| fixture.kind == kind)
        .collect()
}

/// Parse one line from the M1 fixture manifest.
fn parse_fixture_line(line: &str) -> M1Fixture {
    let fields = line.split('|').collect::<Vec<_>>();
    assert_eq!(
        fields.len(),
        8,
        "fixture rows must have eight fields: {line}"
    );

    M1Fixture {
        id: fields[0].to_string(),
        kind: fields[1].to_string(),
        raw_spec: fields[2].to_string(),
        expected_state: fields[3].to_string(),
        expected_reason: optional_field(fields[4]),
        expected_exit_code: fields[5]
            .parse()
            .expect("fixture exit code must be an integer"),
        forwarded_args: parse_forwarded_args(fields[6]),
        sentinel: fields[7].to_string(),
    }
}

/// Convert an empty manifest field into `None`.
fn optional_field(value: &str) -> Option<String> {
    if value.is_empty() {
        return None;
    }

    Some(value.to_string())
}

/// Parse forwarded args from the compact manifest field.
fn parse_forwarded_args(value: &str) -> Vec<String> {
    if value.is_empty() {
        return Vec::new();
    }

    value.split_whitespace().map(str::to_string).collect()
}

#[cfg(test)]
/// Tests for fixture manifest consumption.
mod tests {
    use super::*;
    use crate::{assert_no_network_for_raw_spec, parse_command_intent, M1Reason, PackageSpecParse};

    #[test]
    /// Verifies parser fixtures drive parser behavior.
    fn parser_fixtures_are_consumed_by_parser() {
        for fixture in fixtures_by_kind("parser") {
            let intent = parse_command_intent(&fixture.raw_spec, fixture.forwarded_args.clone());

            match (&fixture.expected_state[..], &intent.package_spec) {
                ("supported", PackageSpecParse::Supported(_)) => {
                    assert_eq!(
                        intent.forwarded_args, fixture.forwarded_args,
                        "{}",
                        fixture.id
                    );
                }
                ("unsupported", PackageSpecParse::Unsupported(unsupported)) => {
                    assert_eq!(
                        Some(unsupported.reason.clone()),
                        expected_reason(&fixture),
                        "{}",
                        fixture.id
                    );
                    assert!(intent.forwarded_args.is_empty(), "{}", fixture.id);
                }
                ("malformed", PackageSpecParse::Malformed(malformed)) => {
                    assert_eq!(
                        Some(malformed.reason.clone()),
                        expected_reason(&fixture),
                        "{}",
                        fixture.id
                    );
                    assert!(intent.forwarded_args.is_empty(), "{}", fixture.id);
                }
                _ => assert!(
                    false,
                    "fixture state did not match parser result: {}",
                    fixture.id
                ),
            }
            assert_eq!(
                parser_exit_code(&intent.package_spec),
                fixture.expected_exit_code,
                "{}",
                fixture.id
            );
        }
    }

    #[test]
    /// Verifies no-network fixtures reach the reusable no-network harness.
    fn no_network_fixtures_cover_malformed_and_unsupported_specs() {
        let no_network = m1_fixtures()
            .into_iter()
            .filter(M1Fixture::expects_no_network)
            .collect::<Vec<_>>();

        assert!(no_network
            .iter()
            .any(|fixture| fixture.expected_state == "malformed"));
        assert!(no_network
            .iter()
            .any(|fixture| fixture.expected_state == "unsupported"));
        for fixture in no_network {
            assert_no_network_for_raw_spec(&fixture.raw_spec);
        }
    }

    #[test]
    /// Verifies registry failure seeds are consumed as explicit outcomes.
    fn registry_failure_fixtures_are_consumed() {
        let registry = fixtures_by_kind("registry");

        assert!(registry
            .iter()
            .any(|fixture| fixture.id == "registry_error"));
        assert!(registry
            .iter()
            .any(|fixture| fixture.id == "missing_package"));
        assert!(registry
            .iter()
            .any(|fixture| fixture.id == "missing_version"));
        for fixture in registry {
            assert_supported_fixture_spec(&fixture);
            assert_seeded_failure_outcome(&fixture, "inspection_error", 3, "no_execution");
        }
    }

    #[test]
    /// Verifies artifact failure seeds are consumed as explicit outcomes.
    fn artifact_failure_fixtures_are_consumed() {
        let artifact = fixtures_by_kind("artifact");

        assert!(artifact
            .iter()
            .any(|fixture| fixture.id == "integrity_mismatch"));
        for fixture in artifact {
            assert_supported_fixture_spec(&fixture);
            assert_seeded_failure_outcome(&fixture, "deny", 4, "no_execution");
        }
    }

    /// Convert fixture reason names into the shared reason enum.
    fn expected_reason(fixture: &M1Fixture) -> Option<M1Reason> {
        match fixture.expected_reason.as_deref() {
            Some("unsupported_spec") => Some(M1Reason::UnsupportedSpec),
            Some("malformed_spec") => Some(M1Reason::MalformedSpec),
            Some("registry_error") => Some(M1Reason::RegistryError),
            Some("missing_package") => Some(M1Reason::MissingPackage),
            Some("missing_version") => Some(M1Reason::MissingVersion),
            Some("integrity_mismatch") => Some(M1Reason::IntegrityMismatch),
            _ => None,
        }
    }

    /// Return the process exit code implied by parser state.
    fn parser_exit_code(parse: &PackageSpecParse) -> i32 {
        match parse {
            PackageSpecParse::Supported(_) => 0,
            PackageSpecParse::Unsupported(_) | PackageSpecParse::Malformed(_) => 2,
        }
    }

    /// Assert a registry or artifact fixture has a supported package spec.
    fn assert_supported_fixture_spec(fixture: &M1Fixture) {
        let intent = parse_command_intent(&fixture.raw_spec, fixture.forwarded_args.clone());

        assert!(
            matches!(intent.package_spec, PackageSpecParse::Supported(_)),
            "{}",
            fixture.id
        );
    }

    /// Assert a registry or artifact fixture seeds an exact failure outcome.
    fn assert_seeded_failure_outcome(
        fixture: &M1Fixture,
        expected_state: &str,
        expected_exit_code: i32,
        expected_sentinel: &str,
    ) {
        let outcome = fixture
            .seeded_failure_outcome()
            .expect("registry and artifact fixtures seed outcomes");

        assert_eq!(outcome.state, expected_state, "{}", fixture.id);
        assert_eq!(outcome.reason, fixture.expected_reason, "{}", fixture.id);
        assert_eq!(outcome.exit_code, expected_exit_code, "{}", fixture.id);
        assert_eq!(
            outcome.exit_code, fixture.expected_exit_code,
            "{}",
            fixture.id
        );
        assert_eq!(outcome.sentinel, expected_sentinel, "{}", fixture.id);
        assert_eq!(outcome.sentinel, fixture.sentinel, "{}", fixture.id);
        assert!(expected_reason(fixture).is_some(), "{}", fixture.id);
    }
}
