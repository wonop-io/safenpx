//! Package metadata parsing helpers for static extraction.

use serde::Deserialize;
use std::collections::BTreeMap;

/// npm bundled dependency declaration shape.
#[derive(Debug, Default, Deserialize)]
#[serde(untagged)]
pub(crate) enum PackageBundledDependencies {
    /// No bundled dependency declaration was present.
    #[default]
    Missing,
    /// Boolean bundled dependency marker.
    Flag(bool),
    /// Array of bundled package names.
    Names(Vec<String>),
    /// Object form used by some package metadata producers.
    Map(BTreeMap<String, String>),
}

impl PackageBundledDependencies {
    /// Convert bundled metadata into dependency declarations.
    pub(crate) fn into_declarations(self) -> BTreeMap<String, String> {
        match self {
            Self::Missing => BTreeMap::new(),
            Self::Flag(enabled) => enabled
                .then(|| ("*".to_string(), "bundled".to_string()))
                .into_iter()
                .collect(),
            Self::Names(names) => names
                .into_iter()
                .map(|name| (name, "bundled".to_string()))
                .collect(),
            Self::Map(map) => map,
        }
    }
}

/// Convert peer dependency metadata values into stable JSON strings.
pub(crate) fn stringify_peer_dependency_metadata(
    peer_dependencies_meta: BTreeMap<String, serde_json::Value>,
) -> BTreeMap<String, String> {
    peer_dependencies_meta
        .into_iter()
        .map(|(name, value)| {
            let requirement =
                serde_json::to_string(&value).expect("peer dependency metadata should serialize");
            (name, requirement)
        })
        .collect()
}
