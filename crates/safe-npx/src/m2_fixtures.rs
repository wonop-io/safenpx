//! M2 closure fixture manifest support.

use crate::{ClosureDecision, M2Reason};
use std::collections::BTreeSet;

/// M2 closure fixture manifest bundled with the crate.
pub const M2_CLOSURE_FIXTURE_MANIFEST: &str =
    include_str!("../fixtures/m2-closure-fixture-manifest.txt");

/// Required M2 fixture kinds.
pub const REQUIRED_M2_FIXTURE_KINDS: &[M2FixtureKind] = &[
    M2FixtureKind::Canary,
    M2FixtureKind::Bin,
    M2FixtureKind::Lifecycle,
    M2FixtureKind::Dependency,
    M2FixtureKind::Registry,
    M2FixtureKind::Race,
    M2FixtureKind::Cache,
    M2FixtureKind::Shim,
    M2FixtureKind::Closure,
];

/// Parsed M2 fixture row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct M2ClosureFixture {
    /// Stable fixture identifier.
    pub id: String,
    /// Closure fixture kind.
    pub kind: M2FixtureKind,
    /// Human-readable fixture purpose.
    pub description: String,
    /// Expected M2 decision.
    pub expected_decision: ClosureDecision,
    /// Expected M2 reason.
    pub expected_reason: M2Reason,
    /// Expected process exit code.
    pub expected_exit_code: i32,
    /// Expected sentinel behavior such as `no_execution`.
    pub sentinel: M2Sentinel,
}

impl M2ClosureFixture {
    /// Return true when this fixture must prove no package code ran.
    pub fn expects_no_execution(&self) -> bool {
        self.sentinel == M2Sentinel::NoExecution
    }
}

/// M2 closure fixture kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum M2FixtureKind {
    /// No-package-code-ran canary fixture.
    Canary,
    /// Bin selection fixture.
    Bin,
    /// Lifecycle-script blocker fixture.
    Lifecycle,
    /// Dependency closure blocker fixture.
    Dependency,
    /// Registry precedence fixture.
    Registry,
    /// Resolution-to-execution race fixture.
    Race,
    /// Cache identity fixture.
    Cache,
    /// Generated shim identity fixture.
    Shim,
    /// General closure proof fixture.
    Closure,
}

/// Expected sentinel behavior for M2 fixtures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum M2Sentinel {
    /// Fixture must not execute package code.
    NoExecution,
}

/// Parse all bundled M2 closure fixtures.
pub fn m2_closure_fixtures() -> Vec<M2ClosureFixture> {
    parse_m2_closure_fixture_manifest(M2_CLOSURE_FIXTURE_MANIFEST)
}

/// Parse an M2 closure fixture manifest.
pub fn parse_m2_closure_fixture_manifest(manifest: &str) -> Vec<M2ClosureFixture> {
    manifest
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(parse_m2_fixture_line)
        .collect()
}

/// Return all fixture kinds present in a fixture slice.
pub fn present_m2_fixture_kinds(fixtures: &[M2ClosureFixture]) -> BTreeSet<M2FixtureKind> {
    fixtures.iter().map(|fixture| fixture.kind).collect()
}

/// Return required M2 fixture kinds not present in a fixture slice.
pub fn missing_m2_fixture_kinds(fixtures: &[M2ClosureFixture]) -> Vec<M2FixtureKind> {
    let present = present_m2_fixture_kinds(fixtures);
    REQUIRED_M2_FIXTURE_KINDS
        .iter()
        .copied()
        .filter(|kind| !present.contains(kind))
        .collect()
}

/// Return an actionable missing-kind failure message.
pub fn missing_m2_fixture_kinds_message(missing: &[M2FixtureKind]) -> String {
    let tokens = missing
        .iter()
        .map(M2FixtureKind::manifest_token)
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "missing required M2 fixture kind(s): {tokens}; add one manifest row for each missing kind"
    )
}

/// Parse one M2 fixture row.
fn parse_m2_fixture_line(line: &str) -> M2ClosureFixture {
    let fields = line.split('|').collect::<Vec<_>>();
    assert_eq!(
        fields.len(),
        7,
        "M2 closure fixture rows must have seven fields: {line}"
    );

    M2ClosureFixture {
        id: required_field(fields[0], "id"),
        kind: parse_kind(fields[1]).expect("M2 fixture kind must be known"),
        description: required_field(fields[2], "description"),
        expected_decision: parse_decision(fields[3]).expect("M2 fixture decision must be known"),
        expected_reason: parse_reason(fields[4]).expect("M2 fixture reason must be known"),
        expected_exit_code: fields[5]
            .parse()
            .expect("M2 fixture exit code must be an integer"),
        sentinel: parse_sentinel(fields[6]).expect("M2 fixture sentinel must be known"),
    }
}

/// Return a required manifest field.
fn required_field(value: &str, name: &str) -> String {
    assert!(
        !value.is_empty(),
        "M2 fixture {name} field must not be empty"
    );
    value.to_string()
}

impl M2FixtureKind {
    /// Return the manifest token for this fixture kind.
    pub fn manifest_token(&self) -> &'static str {
        match self {
            Self::Canary => "canary",
            Self::Bin => "bin",
            Self::Lifecycle => "lifecycle",
            Self::Dependency => "dependency",
            Self::Registry => "registry",
            Self::Race => "race",
            Self::Cache => "cache",
            Self::Shim => "shim",
            Self::Closure => "closure",
        }
    }
}

/// Parse an M2 fixture kind.
fn parse_kind(value: &str) -> Option<M2FixtureKind> {
    match value {
        "canary" => Some(M2FixtureKind::Canary),
        "bin" => Some(M2FixtureKind::Bin),
        "lifecycle" => Some(M2FixtureKind::Lifecycle),
        "dependency" => Some(M2FixtureKind::Dependency),
        "registry" => Some(M2FixtureKind::Registry),
        "race" => Some(M2FixtureKind::Race),
        "cache" => Some(M2FixtureKind::Cache),
        "shim" => Some(M2FixtureKind::Shim),
        "closure" => Some(M2FixtureKind::Closure),
        _ => None,
    }
}

/// Parse an M2 closure decision.
fn parse_decision(value: &str) -> Option<ClosureDecision> {
    match value {
        "allow" => Some(ClosureDecision::Allow),
        "ask" => Some(ClosureDecision::Ask),
        "deny" => Some(ClosureDecision::Deny),
        "unsupported" => Some(ClosureDecision::Unsupported),
        "inspection_error" => Some(ClosureDecision::InspectionError),
        "execution_refused" => Some(ClosureDecision::ExecutionRefused),
        _ => None,
    }
}

/// Parse an M2 closure reason.
fn parse_reason(value: &str) -> Option<M2Reason> {
    match value {
        "interactive_approval_required" => Some(M2Reason::InteractiveApprovalRequired),
        "ambiguous_bin" => Some(M2Reason::AmbiguousBin),
        "missing_bin" => Some(M2Reason::MissingBin),
        "lifecycle_script_present" => Some(M2Reason::LifecycleScriptPresent),
        "unsupported_closure" => Some(M2Reason::UnsupportedClosure),
        "metadata_changed" => Some(M2Reason::MetadataChanged),
        "cache_identity_mismatch" => Some(M2Reason::CacheIdentityMismatch),
        "registry_precedence_mismatch" => Some(M2Reason::RegistryPrecedenceMismatch),
        "shim_identity_mismatch" => Some(M2Reason::ShimIdentityMismatch),
        "non_interactive_stop" => Some(M2Reason::NonInteractiveStop),
        _ => None,
    }
}

/// Parse M2 sentinel behavior.
fn parse_sentinel(value: &str) -> Option<M2Sentinel> {
    match value {
        "no_execution" => Some(M2Sentinel::NoExecution),
        _ => None,
    }
}

#[cfg(test)]
/// Tests for M2 fixture manifest support.
mod tests {
    use super::*;
    use crate::canary_fixtures;

    #[test]
    /// Verifies the bundled M2 manifest is consumed and covers every kind.
    fn bundled_m2_manifest_covers_required_kinds() {
        let fixtures = m2_closure_fixtures();
        let missing = missing_m2_fixture_kinds(&fixtures);

        assert!(
            missing.is_empty(),
            "{}",
            missing_m2_fixture_kinds_message(&missing)
        );
    }

    #[test]
    /// Verifies every row carries a complete golden outcome.
    fn every_m2_fixture_has_golden_outcome() {
        for fixture in m2_closure_fixtures() {
            assert!(!fixture.id.is_empty());
            assert!(!fixture.description.is_empty(), "{}", fixture.id);
            assert!(fixture.expected_exit_code >= 0, "{}", fixture.id);
            assert!(fixture.expects_no_execution(), "{}", fixture.id);
            assert_eq!(
                fixture.expected_reason.refusal_decision(),
                fixture.expected_decision,
                "{}",
                fixture.id
            );
        }
    }

    #[test]
    /// Verifies missing fixture kinds fail with actionable kind names.
    fn missing_fixture_kinds_are_actionable() {
        let fixtures = parse_m2_closure_fixture_manifest(
            "root_binary|canary|only canary|execution_refused|non_interactive_stop|10|no_execution",
        );

        assert_eq!(
            missing_m2_fixture_kinds(&fixtures),
            vec![
                M2FixtureKind::Bin,
                M2FixtureKind::Lifecycle,
                M2FixtureKind::Dependency,
                M2FixtureKind::Registry,
                M2FixtureKind::Race,
                M2FixtureKind::Cache,
                M2FixtureKind::Shim,
                M2FixtureKind::Closure
            ]
        );
        assert_eq!(
            missing_m2_fixture_kinds_message(&missing_m2_fixture_kinds(&fixtures)),
            "missing required M2 fixture kind(s): bin, lifecycle, dependency, registry, race, cache, shim, closure; add one manifest row for each missing kind"
        );
    }

    #[test]
    /// Verifies M2 canary rows correspond to real canary trap fixtures.
    fn canary_rows_are_tied_to_canary_manifest() {
        let canary_ids = canary_fixtures()
            .into_iter()
            .map(|fixture| fixture.id)
            .collect::<BTreeSet<_>>();

        for fixture in m2_closure_fixtures()
            .into_iter()
            .filter(|fixture| fixture.kind == M2FixtureKind::Canary)
        {
            assert!(canary_ids.contains(&fixture.id), "{}", fixture.id);
        }
    }
}
