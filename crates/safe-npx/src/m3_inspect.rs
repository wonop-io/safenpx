//! M3 inspect-mode static extraction pipeline.

use crate::report_optional_evidence::render_package_optional_evidence;
use crate::{
    extract_verified_root_artifact, ArtifactIdentity, ExtractedPackageMetadata, ExtractionError,
};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Static metadata extracted during M3 inspect mode.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StaticExtractionEvidence {
    /// Parsed package metadata tied to the verified artifact.
    pub metadata: ExtractedPackageMetadata,
    /// Size of the verified tarball bytes.
    pub artifact_size_bytes: usize,
    /// Count of regular files inspected from the verified artifact.
    pub file_count: usize,
    /// Human-readable extraction status.
    pub status: &'static str,
}

/// Extract verified bytes into a controlled temporary root for inspect reporting.
pub fn extract_for_inspect(
    artifact_bytes: &[u8],
    artifact_identity: &ArtifactIdentity,
) -> Result<StaticExtractionEvidence, ExtractionError> {
    let extraction_root = inspect_extraction_root(artifact_identity);
    if extraction_root.exists() {
        let _ = std::fs::remove_dir_all(&extraction_root);
    }

    let extracted =
        extract_verified_root_artifact(artifact_bytes, artifact_identity.clone(), &extraction_root);
    let _ = std::fs::remove_dir_all(&extraction_root);

    extracted.map(|extracted| StaticExtractionEvidence {
        metadata: extracted.metadata,
        artifact_size_bytes: extracted.artifact_size_bytes,
        file_count: extracted.file_count,
        status: "extracted",
    })
}

/// Render optional M3 static extraction evidence for human reports.
pub fn render_static_extraction(static_extraction: Option<&StaticExtractionEvidence>) -> String {
    let Some(static_extraction) = static_extraction else {
        return String::new();
    };

    format!(
        "Static extraction: {}\nPackage metadata: {}\nPackage size: {} bytes\nPackage files: {}\nBins: {}\nLifecycle scripts: {}\nDependency declarations: {}\n{}",
        static_extraction.status,
        static_extraction.metadata.package_json_path.display(),
        static_extraction.artifact_size_bytes,
        static_extraction.file_count,
        render_bins(&static_extraction.metadata.bins),
        render_pairs(&static_extraction.metadata.lifecycle_scripts),
        render_dependency_declarations(&static_extraction.metadata.dependency_declarations),
        render_package_optional_evidence(&static_extraction.metadata.optional_evidence)
    )
}

/// Render bin declarations for human inspect output.
fn render_bins(bins: &std::collections::BTreeMap<String, String>) -> String {
    render_pairs(bins)
}

/// Render string maps in deterministic order.
fn render_pairs(values: &std::collections::BTreeMap<String, String>) -> String {
    if values.is_empty() {
        return "none".to_string();
    }

    values
        .iter()
        .map(|(name, value)| format!("{name} -> {value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Render dependency declarations without implying verified closure.
fn render_dependency_declarations(
    declarations: &[crate::ExtractedDependencyDeclaration],
) -> String {
    if declarations.is_empty() {
        return "none".to_string();
    }

    declarations
        .iter()
        .map(|dependency| {
            format!(
                "{} ({:?}) {} [{}]",
                dependency.name,
                dependency.kind,
                dependency.requirement,
                dependency.declaration_status
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Return a per-process extraction root for inspect metadata.
fn inspect_extraction_root(artifact_identity: &ArtifactIdentity) -> PathBuf {
    static NEXT_INSPECT_ROOT_ID: AtomicU64 = AtomicU64::new(0);

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let sequence = NEXT_INSPECT_ROOT_ID.fetch_add(1, Ordering::Relaxed);
    let safe_name = artifact_identity
        .name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();

    std::env::temp_dir().join(format!(
        "safe-npx-inspect-{}-{}-{}-{}-{}",
        std::process::id(),
        unique,
        sequence,
        safe_name,
        artifact_identity.version
    ))
}
