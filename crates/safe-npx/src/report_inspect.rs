//! Shared inspect-model construction and human rendering helpers.

use crate::report_optional_evidence::render_registry_optional_evidence;
use crate::{
    build_authority_context, package_scope_for_parse, redact_report_value, redact_report_values,
    render_static_extraction, CommandIntent, Decision, InspectAuthorityContext, InspectDecision,
    InspectExecutionState, InspectExecutionStateKind, InspectFacts, InspectHeuristic,
    InspectHeuristicKind, InspectModel, InspectNextAction, InspectRefusalFact, InspectRefusalState,
    M1Evidence, M1Reason, PackageSpecParse, SourceContext, UnsupportedSpecCategory,
};

/// Build the shared inspect model from current report facts.
pub(crate) fn build_inspect_model(
    intent: &CommandIntent,
    recommendation: &Decision,
    m1: &M1Evidence,
    source_context: &SourceContext,
) -> InspectModel {
    let (facts, execution_state) = match m1 {
        M1Evidence::NoDownload { reason, downloaded } => (
            InspectFacts {
                command: intent.clone(),
                resolved_package: None,
                registry: None,
                artifact: None,
                root_package: None,
                refusal: Some(InspectRefusalFact {
                    state: InspectRefusalState::NoDownload,
                    reason: reason.clone(),
                    downloaded: *downloaded,
                    detail: None,
                }),
            },
            InspectExecutionStateKind::RefusedBeforeExecution,
        ),
        M1Evidence::Failed {
            reason,
            downloaded,
            detail,
        } => (
            InspectFacts {
                command: intent.clone(),
                resolved_package: None,
                registry: None,
                artifact: None,
                root_package: None,
                refusal: Some(InspectRefusalFact {
                    state: InspectRefusalState::Failed,
                    reason: reason.clone(),
                    downloaded: *downloaded,
                    detail: Some(detail.clone()),
                }),
            },
            InspectExecutionStateKind::FailedBeforeExecution,
        ),
        M1Evidence::Verified {
            resolved_package,
            artifact_identity,
            registry_evidence,
            static_extraction,
            ..
        } => (
            InspectFacts {
                command: intent.clone(),
                resolved_package: Some(resolved_package.clone()),
                registry: Some(registry_evidence.clone()),
                artifact: Some(artifact_identity.clone()),
                root_package: static_extraction.clone(),
                refusal: None,
            },
            InspectExecutionStateKind::StoppedBeforeExecution,
        ),
    };

    InspectModel {
        heuristics: inspect_heuristics(&facts),
        decision: inspect_decision(recommendation, &facts),
        authority_context: inspect_authority_context(&facts, source_context),
        execution_state: InspectExecutionState {
            state: execution_state,
            package_code_executed: false,
        },
        facts,
    }
}

/// Render command intent from the shared model for terminal output.
pub(crate) fn render_model_intent(model: &InspectModel) -> String {
    match &model.facts.command.package_spec {
        PackageSpecParse::Supported(package_spec) => format!(
            "Parsed: {}@{}\nForwarded args: {}\n",
            package_spec.name,
            package_spec.version,
            format_forwarded_args(&model.facts.command.forwarded_args)
        ),
        PackageSpecParse::Unsupported(unsupported) => format!(
            "Rejected: {}\nReason: {}\nCategory: {}\nDownloaded: {}\n",
            redact_report_value(&model.facts.command.requested),
            reason_name(&unsupported.reason),
            unsupported_category_name(&unsupported.category),
            unsupported.downloaded
        ),
        PackageSpecParse::Malformed(malformed) => format!(
            "Rejected: {}\nReason: {}\nDownloaded: {}\n",
            redact_report_value(&model.facts.command.requested),
            reason_name(&malformed.reason),
            malformed.downloaded
        ),
    }
}

/// Render shared inspect model facts for terminal output.
pub(crate) fn render_model_facts(model: &InspectModel) -> String {
    if let Some(refusal) = &model.facts.refusal {
        let detail = refusal
            .detail
            .as_ref()
            .map(|detail| format!("Detail: {}\n", redact_report_value(detail)))
            .unwrap_or_default();
        return format!(
            "M1 evidence: {}\nReason: {}\nDownloaded: {}\n",
            refusal_state_name(&refusal.state),
            reason_name(&refusal.reason),
            refusal.downloaded
        ) + &detail;
    }

    let Some(resolved_package) = &model.facts.resolved_package else {
        return String::new();
    };
    let artifact_identity = model
        .facts
        .artifact
        .as_ref()
        .expect("verified facts should include artifact identity");
    let registry_evidence = model
        .facts
        .registry
        .as_ref()
        .expect("verified facts should include registry evidence");

    format!(
        "M1 evidence: verified\nResolved: {}@{}\nRegistry: {}\nRegistry evidence: {}\nTarball: {}\nIntegrity: {}\nIntegrity metadata: {}\nDigest: {}:{}\n{}{}",
        resolved_package.name,
        resolved_package.version,
        redact_report_value(&resolved_package.registry.url),
        registry_evidence.evidence_boundary,
        redact_report_value(&resolved_package.tarball_url),
        "verified",
        resolved_package.integrity,
        artifact_identity.digest_algorithm,
        artifact_identity.digest,
        render_registry_optional_evidence(registry_evidence),
        render_static_extraction(model.facts.root_package.as_ref())
    )
}

/// Render shared decision, authority, execution, and heuristic model fields.
pub(crate) fn render_model_summary(model: &InspectModel) -> String {
    format!(
        "Recommendation: {:?}\nDecision reasons: {}\nRequired next action: {}\n\n[Authority]\nAuthority: command={}\nsource_context={}\nrunner={}\nactor={}\ncwd={} [{}]\nregistry={}\npackage_scope={}\nAuthority boundary: {}\n\n[Execution]\nExecution: {}; package code executed: {}\n\n[Heuristics]\n{}",
        model.decision.recommendation,
        model.inspect_decision_reasons(),
        next_action_name(&model.decision.required_next_action),
        model.authority_context.redacted.command_intent.display,
        source_context_name(&model.authority_context.redacted.source_context),
        runner_context_name(&model.authority_context.redacted.runner_context),
        actor_context_name(&model.authority_context.redacted.actor_context),
        model.authority_context.redacted.cwd.display,
        model.authority_context.redacted.cwd.category,
        model
            .authority_context
            .redacted
            .registry
            .as_ref()
            .map(format_registry_context)
            .unwrap_or_else(|| "unknown".to_string()),
        model
            .authority_context
            .redacted
            .package_scope
            .as_deref()
            .unwrap_or("unknown"),
        model.authority_context.redacted.sandbox_boundary,
        execution_state_name(&model.execution_state.state),
        model.execution_state.package_code_executed,
        render_model_heuristics(model)
    )
}

/// Build report-only M3 heuristics from extracted facts.
fn inspect_heuristics(facts: &InspectFacts) -> Vec<InspectHeuristic> {
    let mut heuristics = Vec::new();
    if let Some(root_package) = &facts.root_package {
        let metadata = &root_package.metadata;
        if !metadata.lifecycle_scripts.is_empty() {
            heuristics.push(InspectHeuristic {
                kind: InspectHeuristicKind::LifecycleScriptsPresent,
                source: "root_package.package_json",
                message: format!(
                    "{} lifecycle script(s) declared",
                    metadata.lifecycle_scripts.len()
                ),
                report_only: true,
            });
        }
        if !metadata.dependency_declarations.is_empty() {
            heuristics.push(InspectHeuristic {
                kind: InspectHeuristicKind::DependencyDeclarationsPresent,
                source: "root_package.package_json",
                message: format!(
                    "{} dependency declaration(s) are not verified closure",
                    metadata.dependency_declarations.len()
                ),
                report_only: true,
            });
        }
        if metadata.bins.is_empty() {
            heuristics.push(InspectHeuristic {
                kind: InspectHeuristicKind::UnusualPackageShape,
                source: "root_package.package_json",
                message: "no package binary declaration found".to_string(),
                report_only: true,
            });
        }
    }
    heuristics
}

/// Render report-only heuristics from the shared model.
fn render_model_heuristics(model: &InspectModel) -> String {
    if model.heuristics.is_empty() {
        return "none\n".to_string();
    }

    let mut output = String::new();
    for heuristic in &model.heuristics {
        let mode = if heuristic.report_only {
            "report_only"
        } else {
            "policy"
        };
        output.push_str(&format!(
            "- {} [{}] {}: {}\n",
            heuristic_kind_name(&heuristic.kind),
            mode,
            heuristic.source,
            heuristic.message
        ));
    }
    output
}

trait InspectDecisionReasonFormatting {
    fn inspect_decision_reasons(&self) -> String;
}

impl InspectDecisionReasonFormatting for InspectModel {
    fn inspect_decision_reasons(&self) -> String {
        if self.decision.reasons.is_empty() {
            return "none".to_string();
        }

        self.decision.reasons.join(", ")
    }
}

/// Build decision summary without letting M3 heuristics hard-deny execution.
fn inspect_decision(recommendation: &Decision, facts: &InspectFacts) -> InspectDecision {
    let required_next_action = match recommendation {
        Decision::Deny => InspectNextAction::Stop,
        Decision::Allow | Decision::Ask => InspectNextAction::AskUser,
    };
    let reasons = facts
        .refusal
        .as_ref()
        .map(|refusal| vec![reason_name(&refusal.reason).to_string()])
        .unwrap_or_else(|| vec!["m3_heuristics_report_only".to_string()]);

    InspectDecision {
        recommendation: recommendation.clone(),
        reasons,
        required_next_action,
    }
}

/// Build initial authority context; #12 refines redaction later.
fn inspect_authority_context(
    facts: &InspectFacts,
    source_context: &SourceContext,
) -> InspectAuthorityContext {
    let registry_source = facts
        .registry
        .as_ref()
        .map(|registry| registry.registry.clone());

    InspectAuthorityContext {
        redacted: build_authority_context(
            &facts.command.requested,
            source_context,
            registry_source.as_ref(),
            package_scope_for_parse(&facts.command.package_spec),
        ),
    }
}

/// Return the stable serialized name for a refusal state.
fn refusal_state_name(state: &InspectRefusalState) -> &'static str {
    match state {
        InspectRefusalState::NoDownload => "no_download",
        InspectRefusalState::Failed => "failed",
    }
}

/// Return the stable serialized name for a next action.
fn next_action_name(action: &InspectNextAction) -> &'static str {
    match action {
        InspectNextAction::AskUser => "ask_user",
        InspectNextAction::Stop => "stop",
        InspectNextAction::RetryNarrowerCommand => "retry_narrower_command",
    }
}

/// Return the stable serialized name for an execution state.
fn execution_state_name(state: &InspectExecutionStateKind) -> &'static str {
    match state {
        InspectExecutionStateKind::StoppedBeforeExecution => "stopped_before_execution",
        InspectExecutionStateKind::RefusedBeforeExecution => "refused_before_execution",
        InspectExecutionStateKind::FailedBeforeExecution => "failed_before_execution",
    }
}

/// Return the stable serialized name for a heuristic kind.
fn heuristic_kind_name(kind: &InspectHeuristicKind) -> &'static str {
    match kind {
        InspectHeuristicKind::LifecycleScriptsPresent => "lifecycle_scripts_present",
        InspectHeuristicKind::DependencyDeclarationsPresent => "dependency_declarations_present",
        InspectHeuristicKind::UnusualPackageShape => "unusual_package_shape",
    }
}

/// Return the stable serialized name for a caller-declared source context.
fn source_context_name(source_context: &SourceContext) -> &'static str {
    match source_context {
        SourceContext::ManualTerminal => "manual_terminal",
        SourceContext::DocsSnippet => "docs_snippet",
        SourceContext::AgentSkill => "agent_skill",
        SourceContext::Ci => "ci",
        SourceContext::Unknown => "unknown",
    }
}

/// Return the stable serialized name for a runner context.
fn runner_context_name(runner: &crate::AuthorityRunnerContext) -> &'static str {
    match runner {
        crate::AuthorityRunnerContext::LocalTerminal => "local_terminal",
        crate::AuthorityRunnerContext::Ci => "ci",
        crate::AuthorityRunnerContext::Agent => "agent",
        crate::AuthorityRunnerContext::Unknown => "unknown",
    }
}

/// Return the stable serialized name for an actor context.
fn actor_context_name(actor: &crate::AuthorityActorContext) -> &'static str {
    match actor {
        crate::AuthorityActorContext::ManualUser => "manual_user",
        crate::AuthorityActorContext::CodingAgent => "coding_agent",
        crate::AuthorityActorContext::Automation => "automation",
        crate::AuthorityActorContext::Unknown => "unknown",
    }
}

/// Format a redacted registry context without exposing credentials.
fn format_registry_context(registry: &crate::AuthorityRegistryContext) -> String {
    registry
        .scope
        .as_ref()
        .map(|scope| format!("{} ({scope})", registry.display_url))
        .unwrap_or_else(|| registry.display_url.clone())
}

/// Format forwarded CLI arguments for human output.
fn format_forwarded_args(args: &[String]) -> String {
    if args.is_empty() {
        return "[]".to_string();
    }

    redact_report_values(args).join(" ")
}

/// Return the stable serialized name for an M1 reason.
fn reason_name(reason: &M1Reason) -> &'static str {
    match reason {
        M1Reason::UnsupportedSpec => "unsupported_spec",
        M1Reason::MalformedSpec => "malformed_spec",
        M1Reason::RegistryError => "registry_error",
        M1Reason::IntegrityMismatch => "integrity_mismatch",
        M1Reason::MissingPackage => "missing_package",
        M1Reason::MissingVersion => "missing_version",
    }
}

/// Return the stable serialized name for an unsupported spec category.
fn unsupported_category_name(category: &UnsupportedSpecCategory) -> &'static str {
    match category {
        UnsupportedSpecCategory::UnversionedName => "unversioned_name",
        UnsupportedSpecCategory::VersionRange => "version_range",
        UnsupportedSpecCategory::GitUrl => "git_url",
        UnsupportedSpecCategory::LocalPath => "local_path",
        UnsupportedSpecCategory::TarballUrl => "tarball_url",
        UnsupportedSpecCategory::Alias => "alias",
        UnsupportedSpecCategory::MultipleSpecs => "multiple_specs",
        UnsupportedSpecCategory::NpmExecVariant => "npm_exec_variant",
        UnsupportedSpecCategory::Other => "other",
    }
}
