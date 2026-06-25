//! Deterministic M2 package bin selection.
//!
//! Selection is static metadata inspection only. It never invokes package
//! binaries, package managers, lifecycle scripts, or generated shims.

use crate::{ClosureDecision, ExtractedPackageMetadata, M2Reason};
use serde::Serialize;
use std::path::{Component, Path};

/// Deterministically selected package bin metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SelectedPackageBin {
    /// Declared executable command name.
    pub name: String,
    /// Declared path relative to the extracted package root.
    pub relative_path: String,
    /// How the bin was selected.
    pub selection: BinSelectionKind,
}

/// How a package bin candidate was selected.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BinSelectionKind {
    /// Exactly one package `bin` declaration was present.
    SingleDeclaredBin,
}

/// Static bin-selection failure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BinSelectionError {
    /// Conservative M2 decision for this bin-selection failure.
    pub decision: ClosureDecision,
    /// Stable M2 refusal reason.
    pub reason: M2Reason,
    /// Deterministic detail for fixture tests and diagnostics.
    pub detail: String,
}

impl BinSelectionError {
    /// Build an ambiguous-bin selection failure.
    fn ambiguous(bin_names: Vec<String>) -> Self {
        Self {
            decision: M2Reason::AmbiguousBin.refusal_decision(),
            reason: M2Reason::AmbiguousBin,
            detail: format!("multiple package bins declared: {}", bin_names.join(",")),
        }
    }

    /// Build a missing-bin selection failure.
    fn missing() -> Self {
        Self {
            decision: M2Reason::MissingBin.refusal_decision(),
            reason: M2Reason::MissingBin,
            detail: "package declares no executable bin".to_string(),
        }
    }

    /// Build an unsafe-bin-path selection failure.
    fn unsafe_path(path: &str) -> Self {
        Self {
            decision: M2Reason::UnsupportedClosure.refusal_decision(),
            reason: M2Reason::UnsupportedClosure,
            detail: format!("package bin path is not a safe relative path: {path}"),
        }
    }
}

/// Select exactly one package bin from extracted package metadata.
pub fn select_package_bin(
    metadata: &ExtractedPackageMetadata,
) -> Result<SelectedPackageBin, BinSelectionError> {
    match metadata.bins.len() {
        0 => Err(BinSelectionError::missing()),
        1 => {
            let (name, relative_path) = metadata
                .bins
                .iter()
                .next()
                .expect("one bin entry should be present");
            validate_bin_relative_path(relative_path)?;
            Ok(SelectedPackageBin {
                name: name.clone(),
                relative_path: relative_path.clone(),
                selection: BinSelectionKind::SingleDeclaredBin,
            })
        }
        _ => Err(BinSelectionError::ambiguous(
            metadata.bins.keys().cloned().collect(),
        )),
    }
}

/// Validate that a declared package bin path is safe to resolve under package root.
fn validate_bin_relative_path(path: &str) -> Result<(), BinSelectionError> {
    if path.is_empty()
        || path.contains('\0')
        || path.contains('\\')
        || path.contains(':')
        || Path::new(path).is_absolute()
    {
        return Err(BinSelectionError::unsafe_path(path));
    }

    let mut has_normal_component = false;
    for component in Path::new(path).components() {
        match component {
            Component::Normal(_) => has_normal_component = true,
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(BinSelectionError::unsafe_path(path));
            }
        }
    }

    if !has_normal_component {
        return Err(BinSelectionError::unsafe_path(path));
    }

    Ok(())
}

#[cfg(test)]
/// Tests for deterministic M2 bin selection.
mod tests {
    use super::*;
    use crate::{ClosureCommandIdentity, CommandIntent};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    const BIN_SELECTION_FIXTURE_MANIFEST: &str =
        include_str!("../fixtures/bin-selection-fixture-manifest.txt");

    #[test]
    /// Verifies a package with one bin selects that bin deterministically.
    fn single_bin_package_selection_is_deterministic() {
        let selected =
            select_package_bin(&metadata_with_bins([("create-example", "bin/create.js")]))
                .expect("single bin should be selected");

        assert_eq!(selected.name, "create-example");
        assert_eq!(selected.relative_path, "bin/create.js");
        assert_eq!(selected.selection, BinSelectionKind::SingleDeclaredBin);
    }

    #[test]
    /// Verifies ambiguous bins fail closed with a stable reason.
    fn ambiguous_bins_return_stable_refusal() {
        let error = select_package_bin(&metadata_with_bins([
            ("create-example", "bin/create.js"),
            ("other-example", "bin/other.js"),
        ]))
        .expect_err("multiple bins should be ambiguous");

        assert_eq!(error.decision, ClosureDecision::Unsupported);
        assert_eq!(error.reason, M2Reason::AmbiguousBin);
        assert!(error.detail.contains("create-example,other-example"));
    }

    #[test]
    /// Verifies missing bins fail closed with a stable reason.
    fn missing_bin_returns_stable_refusal() {
        let error =
            select_package_bin(&metadata_with_bins([])).expect_err("missing bin should be refused");

        assert_eq!(error.decision, ClosureDecision::Unsupported);
        assert_eq!(error.reason, M2Reason::MissingBin);
        assert_eq!(error.detail, "package declares no executable bin");
    }

    #[test]
    /// Verifies unsafe bin paths fail closed before selected-bin evidence exists.
    fn unsafe_bin_path_returns_stable_refusal() {
        for unsafe_path in [
            "../outside.js",
            "/tmp/outside.js",
            "C:/outside.js",
            "bin\\cli.js",
            "",
        ] {
            let error = select_package_bin(&metadata_with_bins([("create-example", unsafe_path)]))
                .expect_err("unsafe bin path should be refused");

            assert_eq!(error.decision, ClosureDecision::ExecutionRefused);
            assert_eq!(error.reason, M2Reason::UnsupportedClosure);
            assert!(error.detail.contains("not a safe relative path"));
        }
    }

    #[test]
    /// Verifies scoped package bins are selected from normalized metadata.
    fn scoped_package_bin_selection_is_deterministic() {
        let mut metadata = metadata_with_bins([("create-example", "cli.js")]);
        metadata.name = Some("@scope/create-example".to_string());

        let selected = select_package_bin(&metadata).expect("scoped single bin should select");

        assert_eq!(selected.name, "create-example");
        assert_eq!(selected.relative_path, "cli.js");
    }

    #[test]
    /// Verifies package-name/bin-name mismatch is allowed for a single object bin.
    fn package_name_bin_name_mismatch_is_declared_evidence() {
        let mut metadata = metadata_with_bins([("create-other", "bin/create.js")]);
        metadata.name = Some("create-example".to_string());

        let selected = select_package_bin(&metadata).expect("single mismatch bin should select");

        assert_eq!(selected.name, "create-other");
        assert_eq!(selected.relative_path, "bin/create.js");
    }

    #[test]
    /// Verifies forwarded args stay exact in closure command identity JSON.
    fn forwarded_args_are_preserved_in_command_identity_json() {
        let intent = CommandIntent::supported(
            crate::PackageSpec::exact("create-example@1.2.3", "create-example", "1.2.3", None),
            vec![
                "--flag".to_string(),
                "value with spaces".to_string(),
                "--flag".to_string(),
                "".to_string(),
            ],
        );

        let identity = ClosureCommandIdentity::from(&intent);
        let json = serde_json::to_value(&identity).expect("identity should serialize");

        assert_eq!(
            json,
            serde_json::json!({
                "requested": "create-example@1.2.3",
                "forwarded_args": ["--flag", "value with spaces", "--flag", ""]
            })
        );
    }

    #[test]
    /// Verifies the checked-in bin-selection fixture manifest drives selection.
    fn bin_selection_fixture_manifest_is_consumed() {
        let fixtures = parse_bin_selection_fixture_manifest(BIN_SELECTION_FIXTURE_MANIFEST);

        assert_eq!(fixtures.len(), 7);
        for fixture in fixtures {
            let mut metadata = metadata_with_bin_map(fixture.bins);
            metadata.name = Some(fixture.package_name.clone());
            let result = select_package_bin(&metadata);

            match fixture.expected_decision {
                ClosureDecision::Allow => {
                    let selected = result.expect("fixture should select a bin");
                    assert_eq!(
                        selected.name, fixture.expected_bin_name,
                        "{} selected bin name",
                        fixture.id
                    );
                    assert_eq!(
                        selected.relative_path, fixture.expected_bin_path,
                        "{} selected bin path",
                        fixture.id
                    );
                    let intent = CommandIntent::supported(
                        crate::PackageSpec::exact(
                            format!("{}@1.2.3", fixture.package_name),
                            fixture.package_name,
                            "1.2.3",
                            None,
                        ),
                        fixture.forwarded_args.clone(),
                    );
                    assert_eq!(
                        ClosureCommandIdentity::from(&intent).forwarded_args,
                        fixture.forwarded_args,
                        "{} forwarded args",
                        fixture.id
                    );
                }
                ClosureDecision::Unsupported => {
                    let error = result.expect_err("fixture should refuse bin selection");
                    assert_eq!(
                        error.reason,
                        fixture
                            .expected_reason
                            .expect("unsupported fixture needs reason"),
                        "{} refusal reason",
                        fixture.id
                    );
                }
                ClosureDecision::ExecutionRefused => {
                    let error = result.expect_err("fixture should refuse bin selection");
                    assert_eq!(
                        error.reason,
                        fixture
                            .expected_reason
                            .expect("execution-refused fixture needs reason"),
                        "{} refusal reason",
                        fixture.id
                    );
                }
                other => assert!(
                    matches!(
                        other,
                        ClosureDecision::Allow
                            | ClosureDecision::Unsupported
                            | ClosureDecision::ExecutionRefused
                    ),
                    "unsupported fixture decision for {}: {other:?}",
                    fixture.id
                ),
            }
        }
    }

    fn metadata_with_bins<const N: usize>(
        bins: [(&'static str, &'static str); N],
    ) -> ExtractedPackageMetadata {
        metadata_with_bin_map(BTreeMap::from(
            bins.map(|(name, path)| (name.to_string(), path.to_string())),
        ))
    }

    fn metadata_with_bin_map(bins: BTreeMap<String, String>) -> ExtractedPackageMetadata {
        ExtractedPackageMetadata {
            name: Some("create-example".to_string()),
            version: Some("1.2.3".to_string()),
            bins,
            lifecycle_scripts: BTreeMap::new(),
            dependency_declarations: Vec::new(),
            package_json_path: PathBuf::from("package/package.json"),
        }
    }

    #[derive(Debug)]
    struct BinSelectionFixture {
        id: String,
        package_name: String,
        bins: BTreeMap<String, String>,
        forwarded_args: Vec<String>,
        expected_decision: ClosureDecision,
        expected_reason: Option<M2Reason>,
        expected_bin_name: String,
        expected_bin_path: String,
    }

    fn parse_bin_selection_fixture_manifest(manifest: &str) -> Vec<BinSelectionFixture> {
        manifest
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
            .map(parse_bin_selection_fixture)
            .collect()
    }

    fn parse_bin_selection_fixture(line: &str) -> BinSelectionFixture {
        let fields = line.split('|').collect::<Vec<_>>();
        assert_eq!(fields.len(), 8, "fixture row must have 8 fields: {line}");
        BinSelectionFixture {
            id: fields[0].to_string(),
            package_name: fields[1].to_string(),
            bins: parse_bins_field(fields[2]),
            forwarded_args: serde_json::from_str(fields[3])
                .expect("forwarded args fixture should be JSON string array"),
            expected_decision: parse_fixture_decision(fields[4]),
            expected_reason: parse_fixture_reason(fields[5]),
            expected_bin_name: fields[6].to_string(),
            expected_bin_path: fields[7].to_string(),
        }
    }

    fn parse_bins_field(field: &str) -> BTreeMap<String, String> {
        if field.is_empty() {
            return BTreeMap::new();
        }
        field
            .split(',')
            .map(|entry| {
                let (name, path) = entry
                    .split_once('=')
                    .expect("bin fixture entries must be name=path");
                (name.to_string(), path.to_string())
            })
            .collect()
    }

    fn parse_fixture_decision(value: &str) -> ClosureDecision {
        match value {
            "allow" => ClosureDecision::Allow,
            "unsupported" => ClosureDecision::Unsupported,
            "execution_refused" => ClosureDecision::ExecutionRefused,
            other => {
                assert_eq!(other, "allow", "unsupported fixture decision");
                ClosureDecision::Allow
            }
        }
    }

    fn parse_fixture_reason(value: &str) -> Option<M2Reason> {
        match value {
            "none" => None,
            "ambiguous_bin" => Some(M2Reason::AmbiguousBin),
            "missing_bin" => Some(M2Reason::MissingBin),
            "unsupported_closure" => Some(M2Reason::UnsupportedClosure),
            other => {
                assert_eq!(other, "none", "unsupported fixture reason");
                None
            }
        }
    }
}
