//! Expanded deterministic golden tests for M3 inspect JSON.

use crate::{
    build_authority_context_with_paths, render_report, ArtifactIdentity, Cli, CommandIntent,
    Decision, DependencyDeclarationKind, ExtractedDependencyDeclaration, ExtractedPackageMetadata,
    InspectAuthorityContext, InspectDecision, InspectExecutionState, InspectExecutionStateKind,
    InspectFacts, InspectHeuristic, InspectHeuristicKind, InspectModel, InspectNextAction,
    InspectRefusalFact, InspectRefusalState, M1Evidence, M1Reason, PackageOptionalEvidence,
    PackageSpec, RegistryEvidence, RegistrySource, Report, SourceContext, StaticExtractionEvidence,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const INTEGRITY_FAILURE_GOLDEN: &str =
    include_str!("../fixtures/inspect-json-schema-v0-integrity-failure.json");
const STATIC_BLOCKERS_GOLDEN: &str =
    include_str!("../fixtures/inspect-json-schema-v0-static-blockers.json");
const REDACTED_AUTHORITY_GOLDEN: &str =
    include_str!("../fixtures/inspect-json-schema-v0-redacted-authority.json");
const MISSING_OPTIONAL_METADATA_GOLDEN: &str =
    include_str!("../fixtures/inspect-json-schema-v0-missing-optional-metadata.json");

/// Verifies expanded M3 inspect JSON fixtures stay byte-stable.
#[test]
fn expanded_inspect_json_golden_fixtures_are_stable() {
    let cli = Cli::parse_from(["safe-npx", "--json", "create-example@1.2.3"]);
    let integrity_failure = render_schema_golden(&cli, &integrity_failure_report());
    let static_blockers = render_schema_golden(&cli, &static_blockers_report());
    let redacted_authority = render_schema_golden(&cli, &redacted_authority_report());
    let missing_optional_metadata = render_schema_golden(&cli, &missing_optional_metadata_report());

    let cases = [
        (
            "inspect-json-schema-v0-integrity-failure.json",
            integrity_failure.as_str(),
        ),
        (
            "inspect-json-schema-v0-static-blockers.json",
            static_blockers.as_str(),
        ),
        (
            "inspect-json-schema-v0-redacted-authority.json",
            redacted_authority.as_str(),
        ),
        (
            "inspect-json-schema-v0-missing-optional-metadata.json",
            missing_optional_metadata.as_str(),
        ),
    ];

    maybe_print_schema_goldens(&cases);
    if maybe_update_schema_goldens(&cases) {
        return;
    }

    assert_eq!(integrity_failure, INTEGRITY_FAILURE_GOLDEN);
    assert_eq!(static_blockers, STATIC_BLOCKERS_GOLDEN);
    assert_eq!(redacted_authority, REDACTED_AUTHORITY_GOLDEN);
    assert_eq!(missing_optional_metadata, MISSING_OPTIONAL_METADATA_GOLDEN);
}

/// Verifies redacted authority output does not expose fixture secrets or host paths.
#[test]
fn redacted_authority_golden_hides_secret_and_host_inputs() {
    let cli = Cli::parse_from(["safe-npx", "--json", "create-example@1.2.3"]);
    let output = render_schema_golden(&cli, &redacted_authority_report());

    for forbidden in [
        "sekret-token",
        "sekret-auth",
        "/home/example",
        "_authToken=sekret-auth",
    ] {
        assert!(
            !output.contains(forbidden),
            "redacted authority output leaked {forbidden}"
        );
    }
}

/// Verifies schema documentation records compatibility rules enforced by tests.
#[test]
fn schema_docs_record_enum_and_additive_field_compatibility_rules() {
    let schema_doc = schema_doc();
    for required in [
        "Additive fields are allowed within `0.x`.",
        "Enum additions require a schema bump.",
        "Enum semantic changes require a migration note.",
        "During the `0.1` transition",
    ] {
        assert!(
            schema_doc.contains(required),
            "missing doc rule: {required}"
        );
    }
}

/// Renders a report through the public JSON path with a trailing newline.
fn render_schema_golden(cli: &Cli, report: &Report) -> String {
    format!(
        "{}\n",
        render_report(cli, report).expect("schema should render")
    )
}

/// Prints regenerated fixture contents when explicitly requested.
fn maybe_print_schema_goldens(cases: &[(&str, &str)]) {
    if std::env::var_os("SAFE_NPX_PRINT_SCHEMA_GOLDENS").is_none() {
        return;
    }

    for (name, value) in cases {
        eprintln!("--- {name} ---\n{value}");
    }
}

/// Updates fixture files when explicitly requested by a developer.
fn maybe_update_schema_goldens(cases: &[(&str, &str)]) -> bool {
    if std::env::var_os("SAFE_NPX_UPDATE_SCHEMA_GOLDENS").is_none() {
        return false;
    }

    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    for (name, value) in cases {
        std::fs::write(fixture_root.join(name), value).expect("fixture should update");
    }
    true
}

/// Reads the schema doc from Cargo or Bazel test locations.
fn schema_doc() -> String {
    let relative_doc = Path::new("docs/inspect-json-schema-v0.md");
    let mut candidates = vec![PathBuf::from(relative_doc)];
    if let Ok(cwd) = std::env::current_dir() {
        candidates.extend(cwd.ancestors().map(|ancestor| ancestor.join(relative_doc)));
    }
    if let Some(manifest_dir) = std::env::var_os("CARGO_MANIFEST_DIR") {
        candidates.extend(
            PathBuf::from(manifest_dir)
                .ancestors()
                .map(|ancestor| ancestor.join(relative_doc)),
        );
    }
    if let Some(srcdir) = std::env::var_os("TEST_SRCDIR") {
        let srcdir = PathBuf::from(srcdir);
        candidates.push(srcdir.join("_main").join(relative_doc));
        candidates.push(srcdir.join("__main__").join(relative_doc));
        candidates.push(srcdir.join(relative_doc));
    }
    if let (Some(srcdir), Some(workspace)) = (
        std::env::var_os("TEST_SRCDIR"),
        std::env::var_os("TEST_WORKSPACE"),
    ) {
        candidates.push(PathBuf::from(srcdir).join(workspace).join(relative_doc));
    }

    for candidate in candidates {
        if let Ok(contents) = std::fs::read_to_string(&candidate) {
            return contents;
        }
    }

    panic!("could not locate docs/inspect-json-schema-v0.md");
}

/// Builds an integrity-mismatch report that denies execution before extraction.
fn integrity_failure_report() -> Report {
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

/// Builds a verified report with lifecycle and dependency blocker evidence.
fn static_blockers_report() -> Report {
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
fn redacted_authority_report() -> Report {
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
fn missing_optional_metadata_report() -> Report {
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
    Report {
        package_spec: intent.requested.clone(),
        inspect: InspectModel {
            facts,
            heuristics,
            decision: InspectDecision {
                recommendation: recommendation.clone(),
                reasons: vec!["fixture".to_string()],
                required_next_action: InspectNextAction::AskUser,
            },
            authority_context: InspectAuthorityContext {
                redacted: authority_context,
            },
            execution_state: InspectExecutionState {
                state: InspectExecutionStateKind::StoppedBeforeExecution,
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
