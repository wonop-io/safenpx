//! Deterministic M2 package bin selection.
//!
//! Selection is static metadata inspection only. It never invokes package
//! binaries, package managers, lifecycle scripts, or generated shims.

use crate::{ClosureDecision, ExtractedPackageMetadata, M2Reason};
use serde::Serialize;

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

#[cfg(test)]
/// Tests for deterministic M2 bin selection.
mod tests {
    use super::*;
    use crate::{ClosureCommandIdentity, CommandIntent};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

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

    fn metadata_with_bins<const N: usize>(
        bins: [(&'static str, &'static str); N],
    ) -> ExtractedPackageMetadata {
        ExtractedPackageMetadata {
            name: Some("create-example".to_string()),
            version: Some("1.2.3".to_string()),
            bins: BTreeMap::from(bins.map(|(name, path)| (name.to_string(), path.to_string()))),
            lifecycle_scripts: BTreeMap::new(),
            dependency_declarations: Vec::new(),
            package_json_path: PathBuf::from("package/package.json"),
        }
    }
}
