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
    CanaryInspection {
        fixture_id: fixture.id.clone(),
        sentinel_path: fixture.sentinel_path(sentinel_root),
        network_attempt_detected: fixture.network_expectation
            == CanaryNetworkExpectation::DetectedWithoutExecution,
    }
}

/// Inspect canary fixtures and assert that no sentinel was created.
pub fn assert_canary_inspection_leaves_sentinels_absent(
    fixtures: &[CanaryFixture],
    sentinel_root: &Path,
) {
    for fixture in fixtures {
        let inspection = inspect_canary_fixture(fixture, sentinel_root);
        assert!(
            inspection.sentinel_absent(),
            "{} created sentinel {:?}",
            fixture.id,
            inspection.sentinel_path
        );
    }
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

        assert_bundled_canary_inspection_is_safe(workspace.path());

        for fixture in canary_fixtures() {
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
        let inspection = inspect_canary_fixture(&fixture, workspace.path());

        assert!(inspection.network_attempt_detected);
        assert!(inspection.sentinel_absent());
    }

    #[test]
    /// Verifies a touched sentinel would be observable by the harness.
    fn sentinel_presence_is_observable() {
        let workspace = CanaryTempRoot::new();
        let fixture = fixture_by_kind(CanaryTrapKind::RootBinary);
        let sentinel = fixture.sentinel_path(workspace.path());

        fs::write(&sentinel, b"trap ran").expect("test sentinel should be writable");

        let inspection = inspect_canary_fixture(&fixture, workspace.path());
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
