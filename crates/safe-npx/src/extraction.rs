//! Safe static extraction of verified root package artifacts.
//!
//! Extraction only reads tarball bytes and JSON metadata. It never invokes npm,
//! node, package binaries, lifecycle scripts, shell commands, or package
//! managers.

use crate::{ArtifactIdentity, DependencyDeclarationKind};
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Component, Path, PathBuf};
use tar::{Archive, EntryType};

/// Result of extracting a verified root package artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtractedRootArtifact {
    /// M1 artifact identity that authorized these bytes for inspection.
    pub artifact: ArtifactIdentity,
    /// Controlled extraction root used for static inspection.
    pub extraction_root: PathBuf,
    /// Parsed package metadata tied to `artifact`.
    pub metadata: ExtractedPackageMetadata,
}

/// Package metadata needed by M2 static closure checks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtractedPackageMetadata {
    /// Package name from `package.json`.
    pub name: Option<String>,
    /// Package version from `package.json`.
    pub version: Option<String>,
    /// Package binary declarations from `package.json`.
    pub bins: BTreeMap<String, String>,
    /// Lifecycle scripts from `package.json`.
    pub lifecycle_scripts: BTreeMap<String, String>,
    /// Dependency declarations from `package.json`.
    pub dependency_declarations: Vec<ExtractedDependencyDeclaration>,
    /// Path to package metadata relative to the extraction root.
    pub package_json_path: PathBuf,
}

/// Dependency declaration read from package metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtractedDependencyDeclaration {
    /// Dependency name.
    pub name: String,
    /// Declared version/range/source string.
    pub requirement: String,
    /// Dependency declaration kind.
    pub kind: DependencyDeclarationKind,
}

/// Static extraction failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtractionError {
    /// Stable extraction failure reason.
    pub reason: ExtractionErrorReason,
    /// Short diagnostic detail for tests and logs.
    pub detail: String,
}

impl ExtractionError {
    /// Create a new extraction failure.
    fn new(reason: ExtractionErrorReason, detail: impl Into<String>) -> Self {
        Self {
            reason,
            detail: detail.into(),
        }
    }
}

/// Stable reason vocabulary for static extraction failures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtractionErrorReason {
    /// Archive could not be read as a gzip-compressed tarball.
    InvalidArchive,
    /// Archive entry path was unsafe or platform-ambiguous.
    UnsafePath,
    /// Archive link target was unsafe or platform-ambiguous.
    UnsafeLink,
    /// Archive entry type is outside the static inspection surface.
    UnsupportedEntry,
    /// Required package metadata was missing.
    MissingPackageJson,
    /// Package metadata could not be parsed.
    MalformedPackageJson,
}

/// Extract a verified root artifact into a controlled inspection root.
pub fn extract_verified_root_artifact(
    tarball_bytes: &[u8],
    artifact: ArtifactIdentity,
    extraction_root: &Path,
) -> Result<ExtractedRootArtifact, ExtractionError> {
    fs::create_dir_all(extraction_root).map_err(|error| {
        ExtractionError::new(
            ExtractionErrorReason::InvalidArchive,
            format!("could not create extraction root: {error}"),
        )
    })?;

    let decoder = GzDecoder::new(Cursor::new(tarball_bytes));
    let mut archive = Archive::new(decoder);
    let entries = archive.entries().map_err(|error| {
        ExtractionError::new(
            ExtractionErrorReason::InvalidArchive,
            format!("could not read tar entries: {error}"),
        )
    })?;

    for entry in entries {
        let mut entry = entry.map_err(|error| {
            ExtractionError::new(
                ExtractionErrorReason::InvalidArchive,
                format!("could not read tar entry: {error}"),
            )
        })?;
        let relative_path = normalize_archive_path(
            entry
                .path()
                .map_err(|error| {
                    ExtractionError::new(
                        ExtractionErrorReason::UnsafePath,
                        format!("could not read entry path: {error}"),
                    )
                })?
                .as_ref(),
        )?;

        match entry.header().entry_type() {
            EntryType::Directory => {
                fs::create_dir_all(extraction_root.join(&relative_path)).map_err(|error| {
                    ExtractionError::new(
                        ExtractionErrorReason::InvalidArchive,
                        format!("could not create directory {:?}: {error}", relative_path),
                    )
                })?;
            }
            EntryType::Regular => {
                write_regular_entry(&mut entry, extraction_root, &relative_path)?;
            }
            EntryType::Symlink | EntryType::Link => {
                validate_link_target(entry.link_name().map_err(|error| {
                    ExtractionError::new(
                        ExtractionErrorReason::UnsafeLink,
                        format!("could not read link target: {error}"),
                    )
                })?)?;
                return Err(ExtractionError::new(
                    ExtractionErrorReason::UnsupportedEntry,
                    format!("link entry is not extracted: {:?}", relative_path),
                ));
            }
            other => {
                return Err(ExtractionError::new(
                    ExtractionErrorReason::UnsupportedEntry,
                    format!("unsupported archive entry type: {:?}", other),
                ));
            }
        }
    }

    let metadata = read_extracted_package_metadata(extraction_root)?;

    Ok(ExtractedRootArtifact {
        artifact,
        extraction_root: extraction_root.to_path_buf(),
        metadata,
    })
}

/// Write a regular archive entry with validated relative path.
fn write_regular_entry(
    entry: &mut tar::Entry<'_, GzDecoder<Cursor<&[u8]>>>,
    extraction_root: &Path,
    relative_path: &Path,
) -> Result<(), ExtractionError> {
    let destination = extraction_root.join(relative_path);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            ExtractionError::new(
                ExtractionErrorReason::InvalidArchive,
                format!("could not create parent directory {:?}: {error}", parent),
            )
        })?;
    }

    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes).map_err(|error| {
        ExtractionError::new(
            ExtractionErrorReason::InvalidArchive,
            format!("could not read archive file {:?}: {error}", relative_path),
        )
    })?;
    fs::write(&destination, bytes).map_err(|error| {
        ExtractionError::new(
            ExtractionErrorReason::InvalidArchive,
            format!("could not write archive file {:?}: {error}", relative_path),
        )
    })
}

/// Read package metadata from the controlled extraction root.
fn read_extracted_package_metadata(
    extraction_root: &Path,
) -> Result<ExtractedPackageMetadata, ExtractionError> {
    for relative in [Path::new("package/package.json"), Path::new("package.json")] {
        let candidate = extraction_root.join(relative);
        if candidate.exists() {
            let bytes = fs::read(&candidate).map_err(|error| {
                ExtractionError::new(
                    ExtractionErrorReason::MalformedPackageJson,
                    format!("could not read package.json: {error}"),
                )
            })?;
            let package_json: PackageJson = serde_json::from_slice(&bytes).map_err(|error| {
                ExtractionError::new(
                    ExtractionErrorReason::MalformedPackageJson,
                    format!("could not parse package.json: {error}"),
                )
            })?;
            return Ok(package_json.into_metadata(relative.to_path_buf()));
        }
    }

    Err(ExtractionError::new(
        ExtractionErrorReason::MissingPackageJson,
        "package.json was not present in extracted artifact",
    ))
}

/// Normalize and validate an archive path as a relative filesystem path.
fn normalize_archive_path(path: &Path) -> Result<PathBuf, ExtractionError> {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Normal(value) => {
                let text = value.to_string_lossy();
                if text.is_empty() || text.contains('\\') || text.contains(':') {
                    return Err(ExtractionError::new(
                        ExtractionErrorReason::UnsafePath,
                        format!("archive path component is ambiguous: {:?}", value),
                    ));
                }
                normalized.push(value);
            }
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                return Err(ExtractionError::new(
                    ExtractionErrorReason::UnsafePath,
                    format!("archive path is not a safe relative path: {:?}", path),
                ));
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err(ExtractionError::new(
            ExtractionErrorReason::UnsafePath,
            "archive path was empty",
        ));
    }

    Ok(normalized)
}

/// Validate a link target before rejecting link extraction.
fn validate_link_target(target: Option<std::borrow::Cow<'_, Path>>) -> Result<(), ExtractionError> {
    let Some(target) = target else {
        return Err(ExtractionError::new(
            ExtractionErrorReason::UnsafeLink,
            "link entry had no target",
        ));
    };

    normalize_archive_path(target.as_ref()).map_err(|error| {
        ExtractionError::new(
            ExtractionErrorReason::UnsafeLink,
            format!("link target is unsafe: {}", error.detail),
        )
    })?;

    Ok(())
}

/// Deserializable package metadata shape for fields M2 consumes.
#[derive(Debug, Deserialize)]
struct PackageJson {
    name: Option<String>,
    version: Option<String>,
    #[serde(default)]
    bin: PackageBin,
    #[serde(default)]
    scripts: BTreeMap<String, String>,
    #[serde(default)]
    dependencies: BTreeMap<String, String>,
    #[serde(rename = "optionalDependencies", default)]
    optional_dependencies: BTreeMap<String, String>,
    #[serde(rename = "peerDependencies", default)]
    peer_dependencies: BTreeMap<String, String>,
    #[serde(rename = "devDependencies", default)]
    dev_dependencies: BTreeMap<String, String>,
}

impl PackageJson {
    /// Convert deserialized package JSON into extracted metadata.
    fn into_metadata(self, package_json_path: PathBuf) -> ExtractedPackageMetadata {
        let bins = self.bin.into_bins(self.name.as_deref());
        let lifecycle_scripts = self
            .scripts
            .into_iter()
            .filter(|(name, _)| is_lifecycle_script(name))
            .collect();

        let mut dependency_declarations = Vec::new();
        push_dependency_declarations(
            &mut dependency_declarations,
            self.dependencies,
            DependencyDeclarationKind::Runtime,
        );
        push_dependency_declarations(
            &mut dependency_declarations,
            self.optional_dependencies,
            DependencyDeclarationKind::Optional,
        );
        push_dependency_declarations(
            &mut dependency_declarations,
            self.peer_dependencies,
            DependencyDeclarationKind::Peer,
        );
        push_dependency_declarations(
            &mut dependency_declarations,
            self.dev_dependencies,
            DependencyDeclarationKind::Development,
        );

        ExtractedPackageMetadata {
            name: self.name,
            version: self.version,
            bins,
            lifecycle_scripts,
            dependency_declarations,
            package_json_path,
        }
    }
}

/// npm `bin` field shape.
#[derive(Debug, Default, Deserialize)]
#[serde(untagged)]
enum PackageBin {
    /// No `bin` declaration was present.
    #[default]
    Missing,
    /// Single binary path using the package name as command name.
    Single(String),
    /// Named binary map.
    Map(BTreeMap<String, String>),
}

impl PackageBin {
    /// Convert npm `bin` metadata into a stable map.
    fn into_bins(self, package_name: Option<&str>) -> BTreeMap<String, String> {
        match self {
            Self::Missing => BTreeMap::new(),
            Self::Single(path) => package_name
                .map(package_bin_name)
                .map(|name| BTreeMap::from([(name, path)]))
                .unwrap_or_default(),
            Self::Map(map) => map,
        }
    }
}

/// Return the command name implied by a package-name `bin` string.
fn package_bin_name(package_name: &str) -> String {
    package_name
        .rsplit('/')
        .next()
        .unwrap_or(package_name)
        .to_string()
}

/// Return true for lifecycle scripts that can run during install.
fn is_lifecycle_script(name: &str) -> bool {
    matches!(name, "preinstall" | "install" | "postinstall")
}

/// Append dependency metadata for one dependency map.
fn push_dependency_declarations(
    declarations: &mut Vec<ExtractedDependencyDeclaration>,
    dependencies: BTreeMap<String, String>,
    kind: DependencyDeclarationKind,
) {
    declarations.extend(dependencies.into_iter().map(|(name, requirement)| {
        ExtractedDependencyDeclaration {
            name,
            requirement,
            kind: kind.clone(),
        }
    }));
}
