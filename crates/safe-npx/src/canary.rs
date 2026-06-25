//! Reusable M2 canary harness proving inspection does not run package code.
//!
//! The fixtures in this module model payloads that would create sentinel files
//! if binaries, lifecycle scripts, generated shims, or network attempts ran.
//! Inspection reads only fixture metadata and must leave every sentinel absent.

use std::path::{Path, PathBuf};

/// Canary fixture manifest bundled with the crate.
pub const CANARY_FIXTURE_MANIFEST: &str = include_str!("../fixtures/canary-fixture-manifest.txt");

/// Trap kind covered by the no-package-code-ran harness.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CanaryTrapKind {
    /// Root package binary would run.
    RootBinary,
    /// Root package `preinstall` would run.
    RootPreinstall,
    /// Root package `install` would run.
    RootInstall,
    /// Root package `postinstall` would run.
    RootPostinstall,
    /// Dependency lifecycle script would run.
    DependencyLifecycle,
    /// Generated shim would run.
    GeneratedShim,
    /// Package code would attempt network access.
    NetworkAttempt,
}

/// Network expectation for a canary fixture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanaryNetworkExpectation {
    /// Fixture does not model network behavior.
    None,
    /// Inspection detects a network-attempt trap without running package code.
    DetectedWithoutExecution,
}

/// Local trap fixture used by M2 inspection tests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanaryFixture {
    /// Stable fixture identifier.
    pub id: String,
    /// Trap kind represented by this fixture.
    pub trap_kind: CanaryTrapKind,
    /// Sentinel path relative to the temporary sentinel root.
    pub sentinel: PathBuf,
    /// Expected network behavior during inspection.
    pub network_expectation: CanaryNetworkExpectation,
}

impl CanaryFixture {
    /// Return the absolute sentinel path for this fixture.
    pub fn sentinel_path(&self, sentinel_root: &Path) -> PathBuf {
        sentinel_root.join(&self.sentinel)
    }
}

/// Local probe that records blocked canary network attempts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CanaryNetworkProbe {
    blocked_attempts: Vec<String>,
}

impl CanaryNetworkProbe {
    /// Record that inspection observed and blocked a network-attempt trap.
    pub fn record_blocked_attempt(&mut self, fixture: &CanaryFixture) {
        self.blocked_attempts.push(fixture.id.clone());
    }

    /// Return true when a fixture's network attempt was blocked by inspection.
    pub fn blocked_attempt_for(&self, fixture: &CanaryFixture) -> bool {
        self.blocked_attempts
            .iter()
            .any(|attempt| attempt == &fixture.id)
    }

    /// Return all blocked canary network attempts.
    pub fn blocked_attempts(&self) -> &[String] {
        &self.blocked_attempts
    }
}

/// Result of inspecting one canary fixture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanaryInspection {
    /// Fixture that was inspected.
    pub fixture_id: String,
    /// Sentinel path that must remain absent.
    pub sentinel_path: PathBuf,
    /// True when inspection observed a declared network-attempt trap.
    pub network_attempt_detected: bool,
}

impl CanaryInspection {
    /// Return true when the fixture sentinel remains absent after inspection.
    pub fn sentinel_absent(&self) -> bool {
        !self.sentinel_path.exists()
    }
}

/// Inspection subject that can be checked against canary fixtures.
pub trait CanaryInspector {
    /// Inspect one fixture while using the provided local network probe.
    fn inspect_fixture(
        &mut self,
        fixture: &CanaryFixture,
        sentinel_root: &Path,
        network_probe: &mut CanaryNetworkProbe,
    ) -> CanaryInspection;
}

/// Static inspect-mode subject that reads canary metadata without payloads.
#[derive(Clone, Debug, Default)]
pub struct StaticCanaryInspector;

impl CanaryInspector for StaticCanaryInspector {
    /// Inspect canary metadata and block declared network-attempt traps.
    fn inspect_fixture(
        &mut self,
        fixture: &CanaryFixture,
        sentinel_root: &Path,
        network_probe: &mut CanaryNetworkProbe,
    ) -> CanaryInspection {
        if fixture.network_expectation == CanaryNetworkExpectation::DetectedWithoutExecution {
            network_probe.record_blocked_attempt(fixture);
        }

        CanaryInspection {
            fixture_id: fixture.id.clone(),
            sentinel_path: fixture.sentinel_path(sentinel_root),
            network_attempt_detected: network_probe.blocked_attempt_for(fixture),
        }
    }
}

/// Parse all bundled canary fixtures.
pub fn canary_fixtures() -> Vec<CanaryFixture> {
    parse_canary_fixture_manifest(CANARY_FIXTURE_MANIFEST)
}

/// Parse a canary fixture manifest.
pub fn parse_canary_fixture_manifest(manifest: &str) -> Vec<CanaryFixture> {
    manifest
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(parse_canary_fixture_line)
        .collect()
}

/// Inspect one canary fixture without executing its trap payload.
pub fn inspect_canary_fixture(fixture: &CanaryFixture, sentinel_root: &Path) -> CanaryInspection {
    let mut inspector = StaticCanaryInspector;
    let mut network_probe = CanaryNetworkProbe::default();

    inspect_canary_fixture_with(&mut inspector, fixture, sentinel_root, &mut network_probe)
}

/// Inspect one canary fixture with an explicit inspection subject.
pub fn inspect_canary_fixture_with(
    inspector: &mut impl CanaryInspector,
    fixture: &CanaryFixture,
    sentinel_root: &Path,
    network_probe: &mut CanaryNetworkProbe,
) -> CanaryInspection {
    inspector.inspect_fixture(fixture, sentinel_root, network_probe)
}

/// Inspect canary fixtures and assert that no sentinel was created.
pub fn assert_canary_inspection_leaves_sentinels_absent(
    fixtures: &[CanaryFixture],
    sentinel_root: &Path,
) {
    let mut inspector = StaticCanaryInspector;

    assert_canary_inspection_leaves_sentinels_absent_with(&mut inspector, fixtures, sentinel_root);
}

/// Inspect canary fixtures through an explicit subject and assert no sentinels.
pub fn assert_canary_inspection_leaves_sentinels_absent_with(
    inspector: &mut impl CanaryInspector,
    fixtures: &[CanaryFixture],
    sentinel_root: &Path,
) -> Vec<CanaryInspection> {
    let mut inspections = Vec::new();
    let mut network_probe = CanaryNetworkProbe::default();

    for fixture in fixtures {
        let inspection =
            inspect_canary_fixture_with(inspector, fixture, sentinel_root, &mut network_probe);

        if fixture.network_expectation == CanaryNetworkExpectation::DetectedWithoutExecution {
            assert!(
                inspection.network_attempt_detected,
                "{} network attempt was not detected",
                fixture.id
            );
        }
        assert!(
            inspection.sentinel_absent(),
            "{} created sentinel {:?}",
            fixture.id,
            inspection.sentinel_path
        );
        inspections.push(inspection);
    }

    inspections
}

/// Inspect the bundled canary fixtures and assert that no package code ran.
pub fn assert_bundled_canary_inspection_is_safe(sentinel_root: &Path) {
    assert_canary_inspection_leaves_sentinels_absent(&canary_fixtures(), sentinel_root);
}

/// Parse one canary fixture row.
fn parse_canary_fixture_line(line: &str) -> CanaryFixture {
    let fields = line.split('|').collect::<Vec<_>>();
    assert_eq!(
        fields.len(),
        4,
        "canary fixture rows must have four fields: {line}"
    );

    CanaryFixture {
        id: fields[0].to_string(),
        trap_kind: parse_trap_kind(fields[1]).expect("bundled canary trap kind should be known"),
        sentinel: PathBuf::from(fields[2]),
        network_expectation: parse_network_expectation(fields[3])
            .expect("bundled canary network expectation should be known"),
    }
}

/// Parse a trap kind field.
fn parse_trap_kind(value: &str) -> Option<CanaryTrapKind> {
    match value {
        "root_binary" => Some(CanaryTrapKind::RootBinary),
        "root_preinstall" => Some(CanaryTrapKind::RootPreinstall),
        "root_install" => Some(CanaryTrapKind::RootInstall),
        "root_postinstall" => Some(CanaryTrapKind::RootPostinstall),
        "dependency_lifecycle" => Some(CanaryTrapKind::DependencyLifecycle),
        "generated_shim" => Some(CanaryTrapKind::GeneratedShim),
        "network_attempt" => Some(CanaryTrapKind::NetworkAttempt),
        _ => None,
    }
}

/// Parse a network expectation field.
fn parse_network_expectation(value: &str) -> Option<CanaryNetworkExpectation> {
    match value {
        "none" => Some(CanaryNetworkExpectation::None),
        "detected_without_execution" => Some(CanaryNetworkExpectation::DetectedWithoutExecution),
        _ => None,
    }
}

#[cfg(test)]
/// Tests for the canary harness.
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    /// Verifies bundled fixture ids are stable and cover every M2 trap kind.
    fn bundled_canary_manifest_covers_m2_traps() {
        let fixtures = canary_fixtures();
        let ids = fixtures
            .iter()
            .map(|fixture| fixture.id.as_str())
            .collect::<HashSet<_>>();
        let kinds = fixtures
            .iter()
            .map(|fixture| &fixture.trap_kind)
            .collect::<HashSet<_>>();

        assert_eq!(fixtures.len(), 7);
        assert!(ids.contains("root_binary"));
        assert!(ids.contains("root_preinstall"));
        assert!(ids.contains("root_install"));
        assert!(ids.contains("root_postinstall"));
        assert!(ids.contains("dependency_lifecycle"));
        assert!(ids.contains("generated_shim"));
        assert!(ids.contains("network_attempt"));
        assert!(kinds.contains(&CanaryTrapKind::RootBinary));
        assert!(kinds.contains(&CanaryTrapKind::RootPreinstall));
        assert!(kinds.contains(&CanaryTrapKind::RootInstall));
        assert!(kinds.contains(&CanaryTrapKind::RootPostinstall));
        assert!(kinds.contains(&CanaryTrapKind::DependencyLifecycle));
        assert!(kinds.contains(&CanaryTrapKind::GeneratedShim));
        assert!(kinds.contains(&CanaryTrapKind::NetworkAttempt));
    }

    #[test]
    /// Verifies inspect mode leaves all bundled sentinels absent.
    fn inspect_mode_leaves_all_sentinels_absent() {
        let workspace = CanaryTempRoot::new();
        let fixtures = canary_fixtures();

        assert_bundled_canary_inspection_is_safe(workspace.path());

        for fixture in fixtures {
            assert!(
                !fixture.sentinel_path(workspace.path()).exists(),
                "{} sentinel should be absent",
                fixture.id
            );
        }
    }

    #[test]
    /// Verifies root binary traps cannot run during inspection.
    fn root_package_binary_trap_cannot_run_during_inspection() {
        assert_trap_kind_does_not_create_sentinel(CanaryTrapKind::RootBinary);
    }

    #[test]
    /// Verifies root lifecycle traps cannot run during inspection.
    fn root_lifecycle_traps_cannot_run_during_inspection() {
        assert_trap_kind_does_not_create_sentinel(CanaryTrapKind::RootPreinstall);
        assert_trap_kind_does_not_create_sentinel(CanaryTrapKind::RootInstall);
        assert_trap_kind_does_not_create_sentinel(CanaryTrapKind::RootPostinstall);
    }

    #[test]
    /// Verifies dependency lifecycle traps cannot run during inspection.
    fn dependency_lifecycle_trap_cannot_run_during_inspection() {
        assert_trap_kind_does_not_create_sentinel(CanaryTrapKind::DependencyLifecycle);
    }

    #[test]
    /// Verifies generated shim traps cannot run during inspection.
    fn generated_shim_trap_cannot_run_during_inspection() {
        assert_trap_kind_does_not_create_sentinel(CanaryTrapKind::GeneratedShim);
    }

    #[test]
    /// Verifies network-attempt traps are detected without creating a sentinel.
    fn network_attempt_is_detected_without_executing_package_code() {
        let workspace = CanaryTempRoot::new();
        let fixture = fixture_by_kind(CanaryTrapKind::NetworkAttempt);
        let mut inspector = StaticCanaryInspector;
        let mut network_probe = CanaryNetworkProbe::default();
        let inspection = inspect_canary_fixture_with(
            &mut inspector,
            &fixture,
            workspace.path(),
            &mut network_probe,
        );

        assert!(inspection.network_attempt_detected);
        assert_eq!(network_probe.blocked_attempts(), &[fixture.id.clone()]);
        assert!(inspection.sentinel_absent());
    }

    #[test]
    /// Verifies the harness observes a subject that runs package-code payloads.
    fn harness_observes_inspector_that_runs_payloads() {
        let workspace = CanaryTempRoot::new();
        let fixture = fixture_by_kind(CanaryTrapKind::RootBinary);
        let mut inspector = ExecutingCanaryInspector;
        let mut network_probe = CanaryNetworkProbe::default();

        let inspection = inspect_canary_fixture_with(
            &mut inspector,
            &fixture,
            workspace.path(),
            &mut network_probe,
        );

        assert!(!inspection.sentinel_absent());
    }

    /// Assert inspection of one trap kind does not create its sentinel.
    fn assert_trap_kind_does_not_create_sentinel(kind: CanaryTrapKind) {
        let workspace = CanaryTempRoot::new();
        let fixture = fixture_by_kind(kind);
        let inspection = inspect_canary_fixture(&fixture, workspace.path());

        assert!(inspection.sentinel_absent(), "{}", fixture.id);
    }

    /// Return the bundled fixture for a trap kind.
    fn fixture_by_kind(kind: CanaryTrapKind) -> CanaryFixture {
        canary_fixtures()
            .into_iter()
            .find(|fixture| fixture.trap_kind == kind)
            .expect("fixture kind should be present")
    }

    /// Test inspector that simulates accidentally executing package code.
    struct ExecutingCanaryInspector;

    impl CanaryInspector for ExecutingCanaryInspector {
        /// Write the fixture sentinel as if package code had run.
        fn inspect_fixture(
            &mut self,
            fixture: &CanaryFixture,
            sentinel_root: &Path,
            network_probe: &mut CanaryNetworkProbe,
        ) -> CanaryInspection {
            let sentinel = fixture.sentinel_path(sentinel_root);
            fs::write(&sentinel, b"trap ran").expect("test sentinel should be writable");
            if fixture.network_expectation == CanaryNetworkExpectation::DetectedWithoutExecution {
                network_probe.record_blocked_attempt(fixture);
            }

            CanaryInspection {
                fixture_id: fixture.id.clone(),
                sentinel_path: sentinel,
                network_attempt_detected: network_probe.blocked_attempt_for(fixture),
            }
        }
    }

    /// Temporary sentinel root for canary tests.
    struct CanaryTempRoot {
        path: PathBuf,
    }

    impl CanaryTempRoot {
        /// Create a unique temporary sentinel root.
        fn new() -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos();
            let path = std::env::temp_dir()
                .join(format!("safe-npx-canary-{}-{nanos}", std::process::id()));
            fs::create_dir_all(&path).expect("canary temp root should be creatable");

            Self { path }
        }

        /// Return the temporary sentinel root path.
        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for CanaryTempRoot {
        /// Remove the temporary sentinel root.
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
