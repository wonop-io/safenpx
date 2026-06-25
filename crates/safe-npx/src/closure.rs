//! M2 execution-closure evidence contracts.
//!
//! These contracts describe evidence gathered before execution. They do not
//! grant permission to run package code.

use crate::{ArtifactIdentity, CommandIntent, RegistrySource};
use serde::Serialize;

/// Inspect-time execution closure evidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExecutionClosureEvidence {
    /// Command identity requested by the caller.
    pub command: ClosureCommandIdentity,
    /// Verified root artifact identity from M1.
    pub root_artifact: ArtifactIdentity,
    /// Registry source used while building closure evidence.
    pub registry: RegistrySource,
    /// Optional cache evidence for the root artifact.
    pub cache: Option<CacheSource>,
    /// Selected package binary candidate, when deterministic.
    pub selected_bin: Option<ExecutableFileIdentity>,
    /// Generated shim candidate, when modeled.
    pub generated_shim: Option<ExecutableFileIdentity>,
    /// Lifecycle scripts detected in package metadata.
    pub lifecycle_scripts: Vec<LifecycleScript>,
    /// Dependency declarations from package metadata.
    pub dependency_declarations: Vec<DependencyDeclaration>,
    /// Verified dependency artifacts, only after future dependency closure proof.
    pub verified_dependencies: Vec<VerifiedDependencyArtifact>,
    /// Current closure decision for execution.
    pub decision: ClosureDecision,
    /// Stable reasons supporting `decision`.
    pub reasons: Vec<M2Reason>,
}

impl ExecutionClosureEvidence {
    /// Return true only when execution closure evidence is complete enough to run.
    pub fn is_executable(&self) -> bool {
        self.decision == ClosureDecision::Allow
            && self.reasons.is_empty()
            && self.selected_bin.is_some()
            && self.lifecycle_scripts.is_empty()
            && !self.has_unverified_dependency_declarations()
    }

    /// Return true when dependency declarations are not yet verified artifacts.
    pub fn has_unverified_dependency_declarations(&self) -> bool {
        self.dependency_declarations.iter().any(|declaration| {
            !self
                .verified_dependencies
                .iter()
                .any(|artifact| artifact.name == declaration.name)
        })
    }
}

/// Command identity used for closure proof.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ClosureCommandIdentity {
    /// Requested package spec string.
    pub requested: String,
    /// Forwarded arguments preserved exactly after CLI parsing.
    pub forwarded_args: Vec<String>,
}

impl From<&CommandIntent> for ClosureCommandIdentity {
    /// Build command identity from parsed CLI intent.
    fn from(intent: &CommandIntent) -> Self {
        Self {
            requested: intent.requested.clone(),
            forwarded_args: intent.forwarded_args.clone(),
        }
    }
}

/// Local cache evidence used by future closure proof.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CacheSource {
    /// Stable cache namespace, for example `safe-npx`.
    pub namespace: String,
    /// Cache key tied to artifact identity.
    pub key: String,
    /// Digest recorded for the cache entry.
    pub digest: String,
}

/// Executable file identity inside a verified closure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExecutableFileIdentity {
    /// Path relative to the verified extraction root.
    pub relative_path: String,
    /// Digest algorithm used by `digest`.
    pub digest_algorithm: String,
    /// Digest of the executable file or deterministic shim bytes.
    pub digest: String,
    /// Source of this executable identity.
    pub source: ExecutableFileSource,
}

/// Source of executable bytes represented in closure evidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutableFileSource {
    /// File came from the verified root package artifact.
    RootArtifact,
    /// File is a deterministic generated shim.
    GeneratedShim,
    /// File came from a verified dependency artifact.
    DependencyArtifact,
}

/// Lifecycle script metadata detected before execution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LifecycleScript {
    /// Script name such as `postinstall`.
    pub name: String,
    /// Script command as declared in package metadata.
    pub command: String,
}

/// Declared dependency metadata that is not executable proof by itself.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DependencyDeclaration {
    /// Dependency name.
    pub name: String,
    /// Declared version/range/source string.
    pub requirement: String,
    /// Dependency declaration kind.
    pub kind: DependencyDeclarationKind,
}

/// Dependency declaration kind from package metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyDeclarationKind {
    /// `dependencies`.
    Runtime,
    /// `optionalDependencies`.
    Optional,
    /// `peerDependencies`.
    Peer,
    /// `peerDependenciesMeta`.
    PeerMetadata,
    /// `devDependencies`.
    Development,
    /// `bundleDependencies` or `bundledDependencies`.
    Bundled,
}

impl DependencyDeclarationKind {
    /// Return true when M2 needs dependency closure proof for this declaration.
    pub fn requires_m2_dependency_closure(&self) -> bool {
        match self {
            Self::Runtime | Self::Optional | Self::Peer | Self::Bundled => true,
            Self::PeerMetadata | Self::Development => false,
        }
    }
}

/// Verified dependency artifact identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct VerifiedDependencyArtifact {
    /// Dependency package name.
    pub name: String,
    /// Dependency package version.
    pub version: String,
    /// Verified dependency artifact identity.
    pub artifact: ArtifactIdentity,
}

/// M2 closure decision vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClosureDecision {
    /// Evidence and policy permit execution.
    Allow,
    /// A human decision is required.
    Ask,
    /// Known unsafe evidence was found.
    Deny,
    /// Command shape is outside the implemented closure surface.
    Unsupported,
    /// Inspection could not gather reliable closure evidence.
    InspectionError,
    /// Inspection succeeded, but execution closure could not be proven.
    ExecutionRefused,
}

/// Stable M2 reason vocabulary for closure proof.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum M2Reason {
    /// Verified evidence is ready for an interactive human approval prompt.
    InteractiveApprovalRequired,
    /// More than one binary could match the command.
    AmbiguousBin,
    /// No executable binary could be selected.
    MissingBin,
    /// A lifecycle script is present in executable package metadata.
    LifecycleScriptPresent,
    /// Full execution closure cannot be proven.
    UnsupportedClosure,
    /// Metadata changed between inspection and execution preparation.
    MetadataChanged,
    /// Cache entry identity does not match inspected evidence.
    CacheIdentityMismatch,
    /// Registry source selected for execution would differ from inspection.
    RegistryPrecedenceMismatch,
    /// Generated shim identity does not match deterministic evidence.
    ShimIdentityMismatch,
    /// Non-interactive context requires stopping instead of prompting.
    NonInteractiveStop,
}

impl M2Reason {
    /// Return the conservative decision associated with this closure reason.
    pub fn refusal_decision(&self) -> ClosureDecision {
        match self {
            Self::InteractiveApprovalRequired => ClosureDecision::Ask,
            Self::LifecycleScriptPresent
            | Self::UnsupportedClosure
            | Self::MetadataChanged
            | Self::CacheIdentityMismatch
            | Self::RegistryPrecedenceMismatch
            | Self::ShimIdentityMismatch => ClosureDecision::ExecutionRefused,
            Self::AmbiguousBin | Self::MissingBin => ClosureDecision::Unsupported,
            Self::NonInteractiveStop => ClosureDecision::ExecutionRefused,
        }
    }
}

#[cfg(test)]
/// Tests for M2 closure contracts.
mod tests {
    use super::*;

    #[test]
    fn m2_reasons_serialize_as_stable_snake_case() {
        let reasons = [
            M2Reason::InteractiveApprovalRequired,
            M2Reason::AmbiguousBin,
            M2Reason::MissingBin,
            M2Reason::LifecycleScriptPresent,
            M2Reason::UnsupportedClosure,
            M2Reason::MetadataChanged,
            M2Reason::CacheIdentityMismatch,
            M2Reason::RegistryPrecedenceMismatch,
            M2Reason::ShimIdentityMismatch,
            M2Reason::NonInteractiveStop,
        ];

        assert_eq!(
            serde_json::to_value(reasons).expect("M2 reasons should serialize"),
            serde_json::json!([
                "interactive_approval_required",
                "ambiguous_bin",
                "missing_bin",
                "lifecycle_script_present",
                "unsupported_closure",
                "metadata_changed",
                "cache_identity_mismatch",
                "registry_precedence_mismatch",
                "shim_identity_mismatch",
                "non_interactive_stop"
            ])
        );
    }

    #[test]
    fn reasons_map_to_conservative_refusal_decisions() {
        assert_eq!(
            M2Reason::InteractiveApprovalRequired.refusal_decision(),
            ClosureDecision::Ask
        );
        assert_eq!(
            M2Reason::UnsupportedClosure.refusal_decision(),
            ClosureDecision::ExecutionRefused
        );
        assert_eq!(
            M2Reason::LifecycleScriptPresent.refusal_decision(),
            ClosureDecision::ExecutionRefused
        );
        assert_eq!(
            M2Reason::AmbiguousBin.refusal_decision(),
            ClosureDecision::Unsupported
        );
        assert_eq!(
            M2Reason::NonInteractiveStop.refusal_decision(),
            ClosureDecision::ExecutionRefused
        );
    }

    #[test]
    fn dependency_declarations_are_not_verified_artifacts() {
        let evidence = evidence_with(
            Vec::new(),
            vec![DependencyDeclaration {
                name: "left-pad".to_string(),
                requirement: "^1.3.0".to_string(),
                kind: DependencyDeclarationKind::Runtime,
            }],
            Vec::new(),
            ClosureDecision::ExecutionRefused,
            vec![M2Reason::UnsupportedClosure],
        );

        assert!(evidence.has_unverified_dependency_declarations());
        assert!(!evidence.is_executable());
        let json = serde_json::to_string(&evidence).expect("evidence should serialize");
        assert!(json.contains("\"dependency_declarations\""));
        assert!(json.contains("\"verified_dependencies\":[]"));
        assert!(json.contains("\"execution_refused\""));
        assert!(json.contains("\"unsupported_closure\""));
    }

    #[test]
    fn partial_dependency_verification_is_not_executable() {
        let evidence = evidence_with(
            Vec::new(),
            vec![
                DependencyDeclaration {
                    name: "left-pad".to_string(),
                    requirement: "^1.3.0".to_string(),
                    kind: DependencyDeclarationKind::Runtime,
                },
                DependencyDeclaration {
                    name: "right-pad".to_string(),
                    requirement: "^1.0.0".to_string(),
                    kind: DependencyDeclarationKind::Runtime,
                },
            ],
            vec![VerifiedDependencyArtifact {
                name: "left-pad".to_string(),
                version: "1.3.0".to_string(),
                artifact: artifact("left-pad", "1.3.0"),
            }],
            ClosureDecision::Allow,
            Vec::new(),
        );

        assert!(evidence.has_unverified_dependency_declarations());
        assert!(!evidence.is_executable());
    }

    #[test]
    fn executable_requires_complete_allow_evidence() {
        let evidence = evidence_with(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            ClosureDecision::Allow,
            Vec::new(),
        );

        assert!(evidence.is_executable());

        let refused = evidence_with(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            ClosureDecision::Allow,
            vec![M2Reason::UnsupportedClosure],
        );
        assert!(!refused.is_executable());

        let mut root_artifact_only = evidence;
        root_artifact_only.selected_bin = None;
        root_artifact_only.generated_shim = None;
        assert!(!root_artifact_only.is_executable());
    }

    #[test]
    fn evidence_represents_bin_shim_and_lifecycle_without_running() {
        let evidence = evidence_with(
            vec![LifecycleScript {
                name: "postinstall".to_string(),
                command: "node postinstall.js".to_string(),
            }],
            Vec::new(),
            Vec::new(),
            ClosureDecision::ExecutionRefused,
            vec![M2Reason::LifecycleScriptPresent],
        );

        assert_eq!(
            evidence.selected_bin.as_ref().expect("selected bin").source,
            ExecutableFileSource::RootArtifact
        );
        assert_eq!(
            evidence
                .generated_shim
                .as_ref()
                .expect("generated shim")
                .source,
            ExecutableFileSource::GeneratedShim
        );
        assert_eq!(evidence.lifecycle_scripts[0].name, "postinstall");
        assert!(!evidence.is_executable());
    }

    fn evidence_with(
        lifecycle_scripts: Vec<LifecycleScript>,
        dependency_declarations: Vec<DependencyDeclaration>,
        verified_dependencies: Vec<VerifiedDependencyArtifact>,
        decision: ClosureDecision,
        reasons: Vec<M2Reason>,
    ) -> ExecutionClosureEvidence {
        ExecutionClosureEvidence {
            command: ClosureCommandIdentity {
                requested: "create-example@1.2.3".to_string(),
                forwarded_args: vec!["--template".to_string(), "react".to_string()],
            },
            root_artifact: artifact("create-example", "1.2.3"),
            registry: RegistrySource {
                url: "https://registry.npmjs.org/".to_string(),
                scope: None,
            },
            cache: Some(CacheSource {
                namespace: "safe-npx".to_string(),
                key: "sha512-fixture".to_string(),
                digest: "abc123".to_string(),
            }),
            selected_bin: Some(executable(
                "package/bin/create-example.js",
                ExecutableFileSource::RootArtifact,
            )),
            generated_shim: Some(executable(
                ".safe-npx/shims/create-example",
                ExecutableFileSource::GeneratedShim,
            )),
            lifecycle_scripts,
            dependency_declarations,
            verified_dependencies,
            decision,
            reasons,
        }
    }

    fn executable(relative_path: &str, source: ExecutableFileSource) -> ExecutableFileIdentity {
        ExecutableFileIdentity {
            relative_path: relative_path.to_string(),
            digest_algorithm: "sha512".to_string(),
            digest: "abc123".to_string(),
            source,
        }
    }

    fn artifact(name: &str, version: &str) -> ArtifactIdentity {
        ArtifactIdentity {
            name: name.to_string(),
            version: version.to_string(),
            integrity: "sha512-fixture".to_string(),
            digest_algorithm: "sha512".to_string(),
            digest: "abc123".to_string(),
        }
    }
}
