//! Deterministic inspect report fixtures shared by golden tests.

use crate::{
    build_authority_context_with_paths, ArtifactIdentity, CommandIntent, Decision,
    DependencyDeclarationKind, ExtractedDependencyDeclaration, ExtractedPackageMetadata,
    InspectAuthorityContext, InspectDecision, InspectExecutionState, InspectExecutionStateKind,
    InspectFacts, InspectHeuristic, InspectHeuristicKind, InspectModel, InspectNextAction,
    InspectRefusalFact, InspectRefusalState, M1Evidence, M1Reason, PackageOptionalEvidence,
    PackageSpec, RegistryEvidence, RegistrySource, Report, SourceContext, StaticExtractionEvidence,
    UnsupportedSpec,
};
use std::collections::BTreeMap;
use std::path::Path;

/// Builds an integrity-mismatch report that denies execution before extraction.
pub(crate) fn integrity_failure_report() -> Report {
    let intent = supported_intent();
    let facts = InspectFacts {
        command: intent.clone(),
        resolved_package: None,
        registry: None,
        artifact: None,
        root_package: None,
        refusal: Some(InspectRefusalFact {
            state: InspectRefusalState::Failed,
            reason: M1Reason::IntegrityMismatch,
            downloaded: true,
            detail: Some("sha512 digest mismatch".to_string()),
        }),
    };

    report_from_parts(
        intent,
        Decision::Deny,
        facts,
        Vec::new(),
        default_authority_context("create-example@1.2.3"),
        M1Evidence::Failed {
            reason: M1Reason::IntegrityMismatch,
            downloaded: true,
            detail: "sha512 digest mismatch".to_string(),
        },
    )
}

/// Builds a normal verified report with package and registry optional evidence.
pub(crate) fn normal_report() -> Report {
    let intent = supported_intent();
    let artifact = artifact_identity();
    let static_extraction = StaticExtractionEvidence {
        metadata: normal_metadata(),
        artifact_size_bytes: 3072,
        file_count: 5,
        status: "extracted",
    };
    let registry = optional_registry_evidence();
    let facts = InspectFacts {
        command: intent.clone(),
        resolved_package: Some(resolved_package()),
        registry: Some(registry.clone()),
        artifact: Some(artifact.clone()),
        root_package: Some(static_extraction.clone()),
        refusal: None,
    };

    report_from_parts(
        intent,
        Decision::Ask,
        facts,
        Vec::new(),
        default_authority_context("create-example@1.2.3"),
        verified_evidence(artifact, registry, Some(static_extraction)),
    )
}

/// Builds an unsupported-spec report that stops before download.
pub(crate) fn unsupported_report() -> Report {
    let intent = CommandIntent::unsupported(
        "create-example@latest",
        UnsupportedSpec {
            reason: M1Reason::UnsupportedSpec,
            category: crate::UnsupportedSpecCategory::VersionRange,
            downloaded: false,
        },
    );
    let facts = InspectFacts {
        command: intent.clone(),
        resolved_package: None,
        registry: None,
        artifact: None,
        root_package: None,
        refusal: Some(InspectRefusalFact {
            state: InspectRefusalState::NoDownload,
            reason: M1Reason::UnsupportedSpec,
            downloaded: false,
            detail: None,
        }),
    };

    report_from_parts(
        intent,
        Decision::Deny,
        facts,
        Vec::new(),
        default_authority_context("create-example@latest"),
        M1Evidence::NoDownload {
            reason: M1Reason::UnsupportedSpec,
            downloaded: false,
        },
    )
}

/// Builds a verified report with lifecycle and dependency blocker evidence.
pub(crate) fn static_blockers_report() -> Report {
    let intent = supported_intent();
    let artifact = artifact_identity();
    let static_extraction = StaticExtractionEvidence {
        metadata: blocker_metadata(),
        artifact_size_bytes: 4096,
        file_count: 7,
        status: "extracted",
    };
    let registry = registry_evidence();
    let facts = InspectFacts {
        command: intent.clone(),
        resolved_package: Some(resolved_package()),
        registry: Some(registry.clone()),
        artifact: Some(artifact.clone()),
        root_package: Some(static_extraction.clone()),
        refusal: None,
    };

    report_from_parts(
        intent,
        Decision::Ask,
        facts,
        vec![
            InspectHeuristic {
                kind: InspectHeuristicKind::LifecycleScriptsPresent,
                source: "static_root_package_metadata",
                message: "root package declares postinstall".to_string(),
                report_only: true,
            },
            InspectHeuristic {
                kind: InspectHeuristicKind::DependencyDeclarationsPresent,
                source: "static_root_package_metadata",
                message: "root package declares runtime dependencies".to_string(),
                report_only: true,
            },
        ],
        default_authority_context("create-example@1.2.3"),
        verified_evidence(artifact, registry, Some(static_extraction)),
    )
}

/// Builds a verified report with auth-like registry and home path inputs.
pub(crate) fn redacted_authority_report() -> Report {
    let intent = supported_intent();
    let artifact = artifact_identity();
    let registry = RegistrySource {
        url: "https://sekret-token@registry.example.test/npm/?_authToken=sekret-auth".to_string(),
        scope: None,
    };
    let authority = build_authority_context_with_paths(
        &intent.requested,
        &SourceContext::AgentSkill,
        Some(&registry),
        Some("unscoped".to_string()),
        Some(Path::new("/home/example/project")),
        Some(Path::new("/home/example")),
    );
    let registry_evidence = registry_evidence();
    let facts = InspectFacts {
        command: intent.clone(),
        resolved_package: Some(resolved_package()),
        registry: Some(registry_evidence.clone()),
        artifact: Some(artifact.clone()),
        root_package: None,
        refusal: None,
    };

    report_from_parts(
        intent,
        Decision::Ask,
        facts,
        Vec::new(),
        authority,
        verified_evidence(artifact, registry_evidence, None),
    )
}

/// Builds a verified report whose package metadata omits optional evidence.
pub(crate) fn missing_optional_metadata_report() -> Report {
    let intent = supported_intent();
    let artifact = artifact_identity();
    let static_extraction = StaticExtractionEvidence {
        metadata: missing_optional_metadata(),
        artifact_size_bytes: 2048,
        file_count: 3,
        status: "extracted",
    };
    let registry = registry_evidence();
    let facts = InspectFacts {
        command: intent.clone(),
        resolved_package: Some(resolved_package()),
        registry: Some(registry.clone()),
        artifact: Some(artifact.clone()),
        root_package: Some(static_extraction.clone()),
        refusal: None,
    };

    report_from_parts(
        intent,
        Decision::Ask,
        facts,
        Vec::new(),
        default_authority_context("create-example@1.2.3"),
        verified_evidence(artifact, registry, Some(static_extraction)),
    )
}

/// Builds a shared inspect report from deterministic parts.
fn report_from_parts(
    intent: CommandIntent,
    recommendation: Decision,
    facts: InspectFacts,
    heuristics: Vec<InspectHeuristic>,
    authority_context: crate::AuthorityContext,
    m1: M1Evidence,
) -> Report {
    let required_next_action = match recommendation {
        Decision::Deny => InspectNextAction::Stop,
        Decision::Allow | Decision::Ask => InspectNextAction::AskUser,
    };
    let execution_state = facts
        .refusal
        .as_ref()
        .map(|refusal| match refusal.state {
            InspectRefusalState::NoDownload => InspectExecutionStateKind::RefusedBeforeExecution,
            InspectRefusalState::Failed => InspectExecutionStateKind::FailedBeforeExecution,
        })
        .unwrap_or(InspectExecutionStateKind::StoppedBeforeExecution);

    Report {
        package_spec: intent.requested.clone(),
        inspect: InspectModel {
            facts,
            heuristics,
            decision: InspectDecision {
                recommendation: recommendation.clone(),
                reasons: vec!["fixture".to_string()],
                required_next_action,
            },
            authority_context: InspectAuthorityContext {
                redacted: authority_context,
            },
            execution_state: InspectExecutionState {
                state: execution_state,
                package_code_executed: false,
            },
        },
        intent,
        recommendation,
        status: "m3_inspect",
        note: "fixture",
        m1,
    }
}

/// Builds the shared exact-version command intent.
fn supported_intent() -> CommandIntent {
    CommandIntent::supported(
        PackageSpec::exact("create-example@1.2.3", "create-example", "1.2.3", None),
        Vec::new(),
    )
}

/// Builds deterministic artifact identity evidence.
fn artifact_identity() -> ArtifactIdentity {
    ArtifactIdentity {
        name: "create-example".to_string(),
        version: "1.2.3".to_string(),
        integrity: "sha512-example".to_string(),
        digest_algorithm: "sha512".to_string(),
        digest: "digest-example".to_string(),
    }
}

/// Builds deterministic resolved package coordinates.
fn resolved_package() -> crate::ResolvedPackage {
    crate::ResolvedPackage {
        name: "create-example".to_string(),
        version: "1.2.3".to_string(),
        registry: RegistrySource {
            url: crate::PUBLIC_NPM_REGISTRY_URL.to_string(),
            scope: None,
        },
        tarball_url: "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz"
            .to_string(),
        integrity: "sha512-example".to_string(),
    }
}

/// Builds deterministic registry evidence without optional metadata.
fn registry_evidence() -> RegistryEvidence {
    RegistryEvidence {
        registry: RegistrySource {
            url: crate::PUBLIC_NPM_REGISTRY_URL.to_string(),
            scope: None,
        },
        package_scope: crate::PackageScopeCategory::Unscoped,
        name: "create-example".to_string(),
        version: "1.2.3".to_string(),
        publish_time: None,
        maintainers: Vec::new(),
        publisher: None,
        repository: None,
        license: None,
        provenance: BTreeMap::new(),
        dist_integrity: "sha512-example".to_string(),
        tarball_url: "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz"
            .to_string(),
        evidence_boundary: "registry metadata is not proof of tarball package contents",
    }
}

/// Builds deterministic registry evidence with optional metadata present.
fn optional_registry_evidence() -> RegistryEvidence {
    let mut evidence = registry_evidence();
    evidence.publish_time = Some("2026-06-26T07:00:00.000Z".to_string());
    evidence.maintainers = vec![crate::RegistryPerson {
        name: Some("Alice Maintainer".to_string()),
        email: Some("npm-auth-token=metadata-secret".to_string()),
    }];
    evidence.publisher = Some(crate::RegistryPerson {
        name: Some("Publisher".to_string()),
        email: Some("publisher@example.test".to_string()),
    });
    evidence.repository = Some("https://github.com/example/create-example".to_string());
    evidence.license = Some("Apache-2.0".to_string());
    evidence
        .provenance
        .insert("npm-signature".to_string(), "fixture".to_string());
    evidence
}

/// Builds M1 verified evidence with optional static extraction.
fn verified_evidence(
    artifact: ArtifactIdentity,
    registry_evidence: RegistryEvidence,
    static_extraction: Option<StaticExtractionEvidence>,
) -> M1Evidence {
    M1Evidence::Verified {
        resolved_package: resolved_package(),
        integrity_status: "verified",
        artifact_identity: artifact,
        registry_evidence,
        static_extraction,
    }
}

/// Builds deterministic authority context for fixture reports.
fn default_authority_context(command: &str) -> crate::AuthorityContext {
    build_authority_context_with_paths(
        command,
        &SourceContext::Unknown,
        None,
        Some("unscoped".to_string()),
        Some(Path::new("/workspace/safenpx")),
        Some(Path::new("/home/example")),
    )
}

/// Builds normal package metadata with optional evidence present.
fn normal_metadata() -> ExtractedPackageMetadata {
    let mut bins = BTreeMap::new();
    bins.insert(
        "create-example".to_string(),
        "bin/create-example.js".to_string(),
    );
    let mut provenance = BTreeMap::new();
    provenance.insert("source".to_string(), "fixture".to_string());

    ExtractedPackageMetadata {
        name: Some("create-example".to_string()),
        version: Some("1.2.3".to_string()),
        bins,
        lifecycle_scripts: BTreeMap::new(),
        dependency_declarations: Vec::new(),
        optional_evidence: PackageOptionalEvidence {
            repository: Some("https://github.com/example/create-example".to_string()),
            license: Some("Apache-2.0".to_string()),
            maintainers: vec![crate::PackagePerson {
                name: Some("Alice Maintainer".to_string()),
                email: Some("npm-auth-token=metadata-secret".to_string()),
            }],
            provenance,
        },
        package_json_path: Path::new("package/package.json").to_path_buf(),
    }
}

/// Builds package metadata containing lifecycle and dependency blockers.
fn blocker_metadata() -> ExtractedPackageMetadata {
    let mut bins = BTreeMap::new();
    bins.insert(
        "create-example".to_string(),
        "bin/create-example.js".to_string(),
    );
    let mut lifecycle_scripts = BTreeMap::new();
    lifecycle_scripts.insert(
        "postinstall".to_string(),
        "node scripts/postinstall.js".to_string(),
    );

    ExtractedPackageMetadata {
        name: Some("create-example".to_string()),
        version: Some("1.2.3".to_string()),
        bins,
        lifecycle_scripts,
        dependency_declarations: vec![ExtractedDependencyDeclaration {
            name: "left-pad".to_string(),
            requirement: "^1.3.0".to_string(),
            kind: DependencyDeclarationKind::Runtime,
            declaration_status: "declared_not_verified",
        }],
        optional_evidence: PackageOptionalEvidence::default(),
        package_json_path: Path::new("package/package.json").to_path_buf(),
    }
}

/// Builds package metadata with optional evidence intentionally absent.
fn missing_optional_metadata() -> ExtractedPackageMetadata {
    let mut bins = BTreeMap::new();
    bins.insert(
        "create-example".to_string(),
        "bin/create-example.js".to_string(),
    );

    ExtractedPackageMetadata {
        name: Some("create-example".to_string()),
        version: Some("1.2.3".to_string()),
        bins,
        lifecycle_scripts: BTreeMap::new(),
        dependency_declarations: Vec::new(),
        optional_evidence: PackageOptionalEvidence::default(),
        package_json_path: Path::new("package/package.json").to_path_buf(),
    }
}
