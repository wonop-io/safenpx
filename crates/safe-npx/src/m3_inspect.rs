//! M3 inspect-mode static extraction pipeline.

use crate::{
    extract_verified_root_artifact, ArtifactIdentity, ExtractedPackageMetadata, ExtractionError,
};
use serde::Serialize;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Static metadata extracted during M3 inspect mode.
#[derive(Debug, PartialEq, Eq, Serialize)]
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
        "Static extraction: {}\nPackage metadata: {}\nPackage size: {} bytes\nPackage files: {}\nBins: {}\nLifecycle scripts: {}\nDependency declarations: {}\n",
        static_extraction.status,
        static_extraction.metadata.package_json_path.display(),
        static_extraction.artifact_size_bytes,
        static_extraction.file_count,
        static_extraction.metadata.bins.len(),
        static_extraction.metadata.lifecycle_scripts.len(),
        static_extraction.metadata.dependency_declarations.len()
    )
}

/// Return a per-process extraction root for inspect metadata.
fn inspect_extraction_root(artifact_identity: &ArtifactIdentity) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
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
        "safe-npx-inspect-{}-{}-{}-{}",
        std::process::id(),
        unique,
        safe_name,
        artifact_identity.version
    ))
}
