//! Provisional M3 inspect latency budgets and phase evidence.

use serde::Serialize;

/// Cold public-package inspect should stay under five seconds.
pub const COLD_PUBLIC_PACKAGE_INSPECT_BUDGET_MS: u128 = 5_000;
/// Warm fixture-backed inspect should stay under one second.
pub const WARM_FIXTURE_INSPECT_BUDGET_MS: u128 = 1_000;

/// Named latency profile used by docs, tests, and manual measurements.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectLatencyProfile {
    /// Optional live public npm measurement.
    ColdPublicPackage,
    /// Repeatable local fixture-backed measurement.
    WarmFixture,
}

/// Inspect latency phase timings in milliseconds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InspectLatencyPhases {
    /// Registry metadata resolution, JSON parsing, and integrity verification.
    pub resolve_ms: u128,
    /// Tarball byte transport.
    pub download_ms: u128,
    /// Static tarball extraction and package metadata parsing.
    pub extract_ms: u128,
    /// Human or JSON report rendering.
    pub render_ms: u128,
}

impl InspectLatencyPhases {
    /// Return the sum of the tracked inspect phases.
    pub fn total_ms(&self) -> u128 {
        self.resolve_ms + self.download_ms + self.extract_ms + self.render_ms
    }
}

/// Budgeted inspect latency evidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InspectLatencyEvidence {
    /// Measurement profile.
    pub profile: InspectLatencyProfile,
    /// Budget target for the profile.
    pub budget_ms: u128,
    /// Captured phase timings.
    pub phases: InspectLatencyPhases,
    /// Total tracked latency.
    pub total_ms: u128,
    /// Whether the tracked total is inside the budget.
    pub within_budget: bool,
    /// Whether this measured wall-clock profile is enforced in normal CI.
    pub ci_enforced: bool,
    /// Human-readable note about provenance and enforcement.
    pub note: &'static str,
}

/// Build latency evidence for a profile and phase set.
pub fn inspect_latency_evidence(
    profile: InspectLatencyProfile,
    phases: InspectLatencyPhases,
) -> InspectLatencyEvidence {
    let (budget_ms, ci_enforced, note) = match profile {
        InspectLatencyProfile::ColdPublicPackage => (
            COLD_PUBLIC_PACKAGE_INSPECT_BUDGET_MS,
            false,
            "Optional live public-npm measurement; do not enforce in CI.",
        ),
        InspectLatencyProfile::WarmFixture => (
            WARM_FIXTURE_INSPECT_BUDGET_MS,
            false,
            "Fixture-backed measurement suitable for local regression evidence; CI validates the evidence shape and budget constants.",
        ),
    };
    let total_ms = phases.total_ms();

    InspectLatencyEvidence {
        profile,
        budget_ms,
        phases,
        total_ms,
        within_budget: total_ms <= budget_ms,
        ci_enforced,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Records the M3 latency targets from the milestone.
    fn latency_budget_targets_match_m3_milestone() {
        assert_eq!(COLD_PUBLIC_PACKAGE_INSPECT_BUDGET_MS, 5_000);
        assert_eq!(WARM_FIXTURE_INSPECT_BUDGET_MS, 1_000);
    }

    #[test]
    /// Phase evidence totals are stable and budget-aware.
    fn phase_evidence_totals_and_classifies_budget() {
        let evidence = inspect_latency_evidence(
            InspectLatencyProfile::WarmFixture,
            InspectLatencyPhases {
                resolve_ms: 100,
                download_ms: 50,
                extract_ms: 25,
                render_ms: 10,
            },
        );

        assert_eq!(evidence.total_ms, 185);
        assert!(evidence.within_budget);
        assert!(!evidence.ci_enforced);
        assert_eq!(evidence.budget_ms, WARM_FIXTURE_INSPECT_BUDGET_MS);
        assert!(evidence.note.contains("CI validates"));
    }

    #[test]
    /// Live public-package measurements are documented as non-CI evidence.
    fn cold_public_package_budget_is_not_ci_enforced() {
        let evidence = inspect_latency_evidence(
            InspectLatencyProfile::ColdPublicPackage,
            InspectLatencyPhases {
                resolve_ms: 4_000,
                download_ms: 500,
                extract_ms: 250,
                render_ms: 50,
            },
        );

        assert_eq!(evidence.budget_ms, COLD_PUBLIC_PACKAGE_INSPECT_BUDGET_MS);
        assert!(evidence.within_budget);
        assert!(!evidence.ci_enforced);
        assert!(evidence.note.contains("do not enforce in CI"));
    }
}
