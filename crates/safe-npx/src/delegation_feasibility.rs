//! M2 pinned-delegation feasibility fixtures.
//!
//! These fixtures are local analysis data. They do not invoke npm, npx, node,
//! shells, package managers, network access, or third-party package code.

use crate::{ClosureDecision, M2Reason};
use std::collections::BTreeSet;

/// Pinned delegation feasibility manifest bundled with the crate.
pub const DELEGATION_FEASIBILITY_MANIFEST: &str =
    include_str!("../fixtures/delegation-feasibility-manifest.txt");

/// Parsed pinned delegation feasibility row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DelegationFeasibilityRow {
    /// Stable row identifier.
    pub id: String,
    /// Delegation step where byte identity could change.
    pub step: DelegationStep,
    /// Executable or evidence byte source affected by the step.
    pub byte_source: String,
    /// Concrete proof gap for M2.
    pub proof_gap: String,
    /// Expected fail-closed decision.
    pub expected_decision: ClosureDecision,
    /// Expected M2 reason.
    pub expected_reason: M2Reason,
    /// Deterministic local probe or prior fixture backing the row.
    pub probe: DelegationProbe,
}

/// Delegation step that can affect executed bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DelegationStep {
    /// Package metadata or coordinate resolution.
    MetadataResolution,
    /// Registry source selection.
    RegistrySelection,
    /// Package-manager cache selection or mutation.
    CacheSelection,
    /// Dependency resolution and installation.
    DependencyResolution,
    /// Lifecycle script execution.
    LifecycleExecution,
    /// Generated executable shim creation.
    ShimGeneration,
    /// Binary lookup and selection.
    BinSelection,
    /// Package-manager configuration and version behavior.
    Configuration,
    /// Ambient process authority and inherited environment.
    Environment,
}

/// Local probe source backing a delegation feasibility row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DelegationProbe {
    /// Covered by the race matrix fixtures.
    RaceMatrix,
    /// Covered by registry precedence fixtures.
    RegistryPrecedenceFixture,
    /// Covered by static metadata fixtures.
    StaticFixture,
    /// Covered by no-package-code-ran canary fixtures.
    CanaryFixture,
    /// Covered by executable identity fixtures.
    ExecutableIdentityFixture,
    /// Covered by bin-selection fixtures.
    BinSelectionFixture,
}

/// Recommendation produced by the M2 pinned delegation evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DelegationRecommendation {
    /// Do not use pinned package-manager delegation in M2.
    RejectForM2,
}

/// Parse all bundled pinned delegation feasibility rows.
pub fn delegation_feasibility_rows() -> Vec<DelegationFeasibilityRow> {
    parse_delegation_feasibility_manifest(DELEGATION_FEASIBILITY_MANIFEST)
}

/// Parse a pinned delegation feasibility manifest.
pub fn parse_delegation_feasibility_manifest(manifest: &str) -> Vec<DelegationFeasibilityRow> {
    manifest
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(parse_delegation_feasibility_line)
        .collect()
}

/// Return the current M2 recommendation for pinned delegation.
pub fn pinned_delegation_recommendation(
    rows: &[DelegationFeasibilityRow],
) -> DelegationRecommendation {
    assert!(
        rows.iter().all(delegation_row_rejects_for_m2),
        "all M2 pinned delegation rows must fail closed"
    );
    DelegationRecommendation::RejectForM2
}

/// Return every delegation step present in a fixture slice.
pub fn present_delegation_steps(rows: &[DelegationFeasibilityRow]) -> BTreeSet<DelegationStep> {
    rows.iter().map(|row| row.step).collect()
}

/// Return true when a row rejects pinned delegation for M2.
pub fn delegation_row_rejects_for_m2(row: &DelegationFeasibilityRow) -> bool {
    row.expected_decision == ClosureDecision::ExecutionRefused
        && row.expected_reason == M2Reason::UnsupportedClosure
        && !row.proof_gap.is_empty()
        && !row.byte_source.is_empty()
}

fn parse_delegation_feasibility_line(line: &str) -> DelegationFeasibilityRow {
    let fields = line.split('|').collect::<Vec<_>>();
    assert_eq!(
        fields.len(),
        7,
        "delegation feasibility rows must have seven fields: {line}"
    );

    DelegationFeasibilityRow {
        id: required_field(fields[0], "id"),
        step: parse_step(fields[1]).expect("delegation step must be known"),
        byte_source: required_field(fields[2], "byte source"),
        proof_gap: required_field(fields[3], "proof gap"),
        expected_decision: parse_decision(fields[4]).expect("decision must be known"),
        expected_reason: parse_reason(fields[5]).expect("reason must be known"),
        probe: parse_probe(fields[6]).expect("probe must be known"),
    }
}

fn required_field(value: &str, name: &str) -> String {
    assert!(!value.is_empty(), "delegation {name} must not be empty");
    value.to_string()
}

fn parse_step(value: &str) -> Option<DelegationStep> {
    match value {
        "metadata_resolution" => Some(DelegationStep::MetadataResolution),
        "registry_selection" => Some(DelegationStep::RegistrySelection),
        "cache_selection" => Some(DelegationStep::CacheSelection),
        "dependency_resolution" => Some(DelegationStep::DependencyResolution),
        "lifecycle_execution" => Some(DelegationStep::LifecycleExecution),
        "shim_generation" => Some(DelegationStep::ShimGeneration),
        "bin_selection" => Some(DelegationStep::BinSelection),
        "configuration" => Some(DelegationStep::Configuration),
        "environment" => Some(DelegationStep::Environment),
        _ => None,
    }
}

fn parse_decision(value: &str) -> Option<ClosureDecision> {
    match value {
        "execution_refused" => Some(ClosureDecision::ExecutionRefused),
        _ => None,
    }
}

fn parse_reason(value: &str) -> Option<M2Reason> {
    match value {
        "unsupported_closure" => Some(M2Reason::UnsupportedClosure),
        _ => None,
    }
}

fn parse_probe(value: &str) -> Option<DelegationProbe> {
    match value {
        "race_matrix" => Some(DelegationProbe::RaceMatrix),
        "registry_precedence_fixture" => Some(DelegationProbe::RegistryPrecedenceFixture),
        "static_fixture" => Some(DelegationProbe::StaticFixture),
        "canary_fixture" => Some(DelegationProbe::CanaryFixture),
        "executable_identity_fixture" => Some(DelegationProbe::ExecutableIdentityFixture),
        "bin_selection_fixture" => Some(DelegationProbe::BinSelectionFixture),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_manifest_covers_delegation_byte_change_steps() {
        let rows = delegation_feasibility_rows();
        let steps = present_delegation_steps(&rows);

        for step in [
            DelegationStep::MetadataResolution,
            DelegationStep::RegistrySelection,
            DelegationStep::CacheSelection,
            DelegationStep::DependencyResolution,
            DelegationStep::LifecycleExecution,
            DelegationStep::ShimGeneration,
            DelegationStep::BinSelection,
            DelegationStep::Configuration,
            DelegationStep::Environment,
        ] {
            assert!(steps.contains(&step), "{step:?}");
        }
    }

    #[test]
    fn every_delegation_gap_refuses_without_fallback() {
        for row in delegation_feasibility_rows() {
            assert!(delegation_row_rejects_for_m2(&row), "{}", row.id);
        }
    }

    #[test]
    fn recommendation_rejects_pinned_delegation_for_m2() {
        assert_eq!(
            pinned_delegation_recommendation(&delegation_feasibility_rows()),
            DelegationRecommendation::RejectForM2
        );
    }

    #[test]
    fn probes_are_deterministic_local_fixture_sources() {
        for row in delegation_feasibility_rows() {
            assert!(
                matches!(
                    row.probe,
                    DelegationProbe::RaceMatrix
                        | DelegationProbe::RegistryPrecedenceFixture
                        | DelegationProbe::StaticFixture
                        | DelegationProbe::CanaryFixture
                        | DelegationProbe::ExecutableIdentityFixture
                        | DelegationProbe::BinSelectionFixture
                ),
                "{}",
                row.id
            );
        }
    }
}
