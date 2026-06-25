//! Deterministic M2 resolution-to-execution race fixtures.
//!
//! The race matrix is local fixture data. It never reaches live npm and never
//! invokes package code, package managers, shells, lifecycle scripts, or node.

use crate::{ClosureDecision, M1Reason, M2Reason};
use std::collections::BTreeSet;

/// Race matrix fixture manifest bundled with the crate.
pub const RACE_MATRIX_FIXTURE_MANIFEST: &str =
    include_str!("../fixtures/race-matrix-fixture-manifest.txt");

/// Parsed resolution-to-execution race fixture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RaceMatrixFixture {
    /// Stable fixture identifier.
    pub id: String,
    /// Race class represented by the fixture.
    pub race_class: RaceClass,
    /// Execution mechanism decision under evaluation.
    pub execution_decision: RaceExecutionDecision,
    /// Inspected identity token.
    pub inspected: String,
    /// Prepared identity token.
    pub prepared: String,
    /// Expected closure or M1 decision.
    pub expected_decision: ClosureDecision,
    /// Expected stable reason.
    pub expected_reason: RaceReason,
    /// Expected sentinel behavior.
    pub sentinel: RaceSentinel,
}

impl RaceMatrixFixture {
    /// Return true when the fixture models identity drift.
    pub fn has_identity_drift(&self) -> bool {
        self.inspected != self.prepared
    }

    /// Return true when exact-version execution remains pinned.
    pub fn exact_version_is_pinned(&self) -> bool {
        self.race_class == RaceClass::ExactVersion && !self.has_identity_drift()
    }
}

/// Resolution-to-execution race class.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RaceClass {
    /// Registry metadata changed after inspection.
    Metadata,
    /// Floating dist-tag moved after inspection.
    Tag,
    /// Cache contents or cache key identity mismatched.
    Cache,
    /// Tarball bytes differed from registry integrity.
    Tarball,
    /// Delegated execution cannot prove it will run inspected bytes.
    Delegation,
    /// Exact-version path is pinned and does not re-resolve.
    ExactVersion,
}

/// Execution mechanism decision covered by a race fixture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RaceExecutionDecision {
    /// Direct execution from verified extracted bytes.
    DirectExtract,
    /// Delegation to another tool with pinned identity requirements.
    PinnedDelegation,
    /// Inspect-only alpha mode.
    InspectOnlyAlpha,
}

/// Stable race reason spanning M2 closure and M1 integrity outcomes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RaceReason {
    /// M2 closure proof reason.
    M2(M2Reason),
    /// M1 artifact verification reason.
    M1(M1Reason),
}

/// Expected sentinel behavior for race fixtures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RaceSentinel {
    /// Fixture must not execute package code.
    NoExecution,
}

/// Parse bundled race matrix fixtures.
pub fn race_matrix_fixtures() -> Vec<RaceMatrixFixture> {
    parse_race_matrix_fixture_manifest(RACE_MATRIX_FIXTURE_MANIFEST)
}

/// Parse a race matrix fixture manifest.
pub fn parse_race_matrix_fixture_manifest(manifest: &str) -> Vec<RaceMatrixFixture> {
    manifest
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(parse_race_fixture_line)
        .collect()
}

/// Return all race classes present in a fixture slice.
pub fn present_race_classes(fixtures: &[RaceMatrixFixture]) -> BTreeSet<RaceClass> {
    fixtures.iter().map(|fixture| fixture.race_class).collect()
}

/// Evaluate whether a fixture carries the expected fail-closed outcome.
pub fn race_fixture_outcome_is_consistent(fixture: &RaceMatrixFixture) -> bool {
    match (&fixture.race_class, &fixture.expected_reason) {
        (RaceClass::Metadata | RaceClass::Tag, RaceReason::M2(M2Reason::MetadataChanged)) => {
            fixture.expected_decision == ClosureDecision::ExecutionRefused
        }
        (RaceClass::Cache, RaceReason::M2(M2Reason::CacheIdentityMismatch)) => {
            fixture.expected_decision == ClosureDecision::ExecutionRefused
        }
        (RaceClass::Tarball, RaceReason::M1(M1Reason::IntegrityMismatch)) => {
            fixture.expected_decision == ClosureDecision::Deny
        }
        (RaceClass::Delegation, RaceReason::M2(M2Reason::UnsupportedClosure)) => {
            fixture.expected_decision == ClosureDecision::ExecutionRefused
        }
        (RaceClass::ExactVersion, RaceReason::M2(M2Reason::InteractiveApprovalRequired)) => {
            fixture.expected_decision == ClosureDecision::Ask
        }
        _ => false,
    }
}

/// Parse one race fixture row.
fn parse_race_fixture_line(line: &str) -> RaceMatrixFixture {
    let fields = line.split('|').collect::<Vec<_>>();
    assert_eq!(
        fields.len(),
        8,
        "race matrix fixture rows must have eight fields: {line}"
    );

    RaceMatrixFixture {
        id: required_field(fields[0], "id"),
        race_class: parse_race_class(fields[1]).expect("race class must be known"),
        execution_decision: parse_execution_decision(fields[2])
            .expect("execution decision must be known"),
        inspected: required_field(fields[3], "inspected"),
        prepared: required_field(fields[4], "prepared"),
        expected_decision: parse_decision(fields[5]).expect("race decision must be known"),
        expected_reason: parse_race_reason(fields[6]).expect("race reason must be known"),
        sentinel: parse_sentinel(fields[7]).expect("race sentinel must be known"),
    }
}

/// Return a required fixture field.
fn required_field(value: &str, name: &str) -> String {
    assert!(!value.is_empty(), "race fixture {name} must not be empty");
    value.to_string()
}

/// Parse a race class token.
fn parse_race_class(value: &str) -> Option<RaceClass> {
    match value {
        "metadata" => Some(RaceClass::Metadata),
        "tag" => Some(RaceClass::Tag),
        "cache" => Some(RaceClass::Cache),
        "tarball" => Some(RaceClass::Tarball),
        "delegation" => Some(RaceClass::Delegation),
        "exact_version" => Some(RaceClass::ExactVersion),
        _ => None,
    }
}

/// Parse an execution decision token.
fn parse_execution_decision(value: &str) -> Option<RaceExecutionDecision> {
    match value {
        "direct_extract" => Some(RaceExecutionDecision::DirectExtract),
        "pinned_delegation" => Some(RaceExecutionDecision::PinnedDelegation),
        "inspect_only_alpha" => Some(RaceExecutionDecision::InspectOnlyAlpha),
        _ => None,
    }
}

/// Parse a closure decision token.
fn parse_decision(value: &str) -> Option<ClosureDecision> {
    match value {
        "ask" => Some(ClosureDecision::Ask),
        "deny" => Some(ClosureDecision::Deny),
        "execution_refused" => Some(ClosureDecision::ExecutionRefused),
        _ => None,
    }
}

/// Parse a race reason token.
fn parse_race_reason(value: &str) -> Option<RaceReason> {
    match value {
        "interactive_approval_required" => {
            Some(RaceReason::M2(M2Reason::InteractiveApprovalRequired))
        }
        "metadata_changed" => Some(RaceReason::M2(M2Reason::MetadataChanged)),
        "cache_identity_mismatch" => Some(RaceReason::M2(M2Reason::CacheIdentityMismatch)),
        "unsupported_closure" => Some(RaceReason::M2(M2Reason::UnsupportedClosure)),
        "integrity_mismatch" => Some(RaceReason::M1(M1Reason::IntegrityMismatch)),
        _ => None,
    }
}

/// Parse sentinel behavior.
fn parse_sentinel(value: &str) -> Option<RaceSentinel> {
    match value {
        "no_execution" => Some(RaceSentinel::NoExecution),
        _ => None,
    }
}

#[cfg(test)]
/// Tests for resolution-to-execution race fixtures.
mod tests {
    use super::*;

    #[test]
    /// Verifies the bundled race matrix covers required race classes.
    fn bundled_race_matrix_covers_required_classes() {
        let fixtures = race_matrix_fixtures();
        let classes = present_race_classes(&fixtures);

        assert!(classes.contains(&RaceClass::Metadata));
        assert!(classes.contains(&RaceClass::Tag));
        assert!(classes.contains(&RaceClass::Cache));
        assert!(classes.contains(&RaceClass::Tarball));
        assert!(classes.contains(&RaceClass::Delegation));
        assert!(classes.contains(&RaceClass::ExactVersion));
    }

    #[test]
    /// Verifies each fixture carries a fail-closed or pinned exact-version outcome.
    fn every_race_fixture_has_consistent_outcome() {
        for fixture in race_matrix_fixtures() {
            assert!(
                race_fixture_outcome_is_consistent(&fixture),
                "{}",
                fixture.id
            );
            assert_eq!(
                fixture.sentinel,
                RaceSentinel::NoExecution,
                "{}",
                fixture.id
            );
        }
    }

    #[test]
    /// Verifies metadata changes after inspection fail closed.
    fn metadata_changed_after_inspection_fails_closed() {
        let fixture = fixture("metadata_changed");

        assert!(fixture.has_identity_drift());
        assert_eq!(fixture.expected_decision, ClosureDecision::ExecutionRefused);
        assert_eq!(
            fixture.expected_reason,
            RaceReason::M2(M2Reason::MetadataChanged)
        );
    }

    #[test]
    /// Verifies tag movement is a future latest blocker while exact versions pin.
    fn tag_movement_blocks_latest_but_exact_version_pins() {
        let tag = fixture("tag_moved_latest");
        let exact = fixture("exact_version_pinned");

        assert!(tag.has_identity_drift());
        assert_eq!(
            tag.execution_decision,
            RaceExecutionDecision::InspectOnlyAlpha
        );
        assert_eq!(
            tag.expected_reason,
            RaceReason::M2(M2Reason::MetadataChanged)
        );
        assert!(exact.exact_version_is_pinned());
        assert_eq!(exact.expected_decision, ClosureDecision::Ask);
    }

    #[test]
    /// Verifies cache races fail closed with cache identity mismatch.
    fn cache_races_fail_closed() {
        for fixture_id in ["cache_poisoned", "stale_cache_reuse"] {
            let fixture = fixture(fixture_id);

            assert_eq!(fixture.race_class, RaceClass::Cache);
            assert_eq!(
                fixture.expected_reason,
                RaceReason::M2(M2Reason::CacheIdentityMismatch)
            );
            assert_eq!(fixture.expected_decision, ClosureDecision::ExecutionRefused);
        }
    }

    #[test]
    /// Verifies tarball identity drift maps to the existing integrity mismatch.
    fn tarball_identity_race_maps_to_integrity_mismatch() {
        let fixture = fixture("tarball_identity_mismatch");

        assert!(fixture.has_identity_drift());
        assert_eq!(fixture.race_class, RaceClass::Tarball);
        assert_eq!(
            fixture.expected_reason,
            RaceReason::M1(M1Reason::IntegrityMismatch)
        );
        assert_eq!(fixture.expected_decision, ClosureDecision::Deny);
    }

    #[test]
    /// Verifies pinned delegation is blocked until it proves inspected bytes.
    fn pinned_delegation_without_byte_proof_fails_closed() {
        let fixture = fixture("pinned_delegation_unproven");

        assert_eq!(
            fixture.execution_decision,
            RaceExecutionDecision::PinnedDelegation
        );
        assert_eq!(
            fixture.expected_reason,
            RaceReason::M2(M2Reason::UnsupportedClosure)
        );
        assert_eq!(fixture.expected_decision, ClosureDecision::ExecutionRefused);
    }

    fn fixture(id: &str) -> RaceMatrixFixture {
        race_matrix_fixtures()
            .into_iter()
            .find(|fixture| fixture.id == id)
            .expect("fixture should exist")
    }
}
