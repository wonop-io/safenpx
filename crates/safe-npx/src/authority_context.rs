//! Authority-context classification and redaction for inspect reports.

use crate::{
    digest_report_key, redact_report_value, redact_report_value_for_home, PackageSpecParse,
    RegistrySource, SourceContext,
};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Redacted authority context carried by inspect reports.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AuthorityContext {
    /// Redacted command/package intent.
    pub command_intent: AuthorityDisplayValue,
    /// Caller-declared source context, or unknown when undeclared.
    pub source_context: SourceContext,
    /// Runner category derived from caller declaration.
    pub runner_context: AuthorityRunnerContext,
    /// Actor category derived from caller declaration.
    pub actor_context: AuthorityActorContext,
    /// Current-directory category and redacted display.
    pub cwd: AuthorityDisplayValue,
    /// Registry category and redacted display when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<AuthorityRegistryContext>,
    /// Package scope category when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_scope: Option<String>,
    /// Separate identity placeholder for later receipt/cache-key work.
    pub identity: AuthorityIdentityContext,
    /// Explicit reminder that authority context is not sandboxing.
    pub sandbox_boundary: &'static str,
}

/// Redacted display value plus category.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AuthorityDisplayValue {
    /// Category suitable for human and JSON output.
    pub category: String,
    /// Redacted display string.
    pub display: String,
}

/// Registry authority context.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AuthorityRegistryContext {
    /// Registry source category.
    pub category: AuthorityRegistryCategory,
    /// Redacted registry URL display.
    pub display_url: String,
    /// Scope that selected this registry, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Placeholder for future canonical receipt/cache identity fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AuthorityIdentityContext {
    /// Stable note explaining that display strings are not identity keys.
    pub status: &'static str,
    /// Canonical command identity with secrets and local paths removed.
    pub command_intent_key: String,
    /// Canonical cwd trust class used before receipt semantics exist.
    pub cwd_trust_class: String,
    /// Canonical registry identity without credentials.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry_key: Option<String>,
}

/// Runner context category.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityRunnerContext {
    /// Human-declared local terminal runner.
    LocalTerminal,
    /// CI runner.
    Ci,
    /// Coding-agent runner.
    Agent,
    /// Runner context is unknown.
    Unknown,
}

/// Actor context category.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityActorContext {
    /// Manual user.
    ManualUser,
    /// Coding agent.
    CodingAgent,
    /// Automation or CI actor.
    Automation,
    /// Actor context is unknown.
    Unknown,
}

/// Registry source category.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityRegistryCategory {
    /// Public npm registry.
    PublicNpm,
    /// Scoped registry selected by package scope.
    ScopedRegistry,
    /// Custom unscoped registry.
    CustomRegistry,
    /// Registry source is unknown.
    Unknown,
}

/// Build redacted authority context for an inspect report.
pub fn build_authority_context(
    command: &str,
    source_context: &SourceContext,
    registry: Option<&RegistrySource>,
    package_scope: Option<String>,
) -> AuthorityContext {
    build_authority_context_with_paths(
        command,
        source_context,
        registry,
        package_scope,
        std::env::current_dir().ok().as_deref(),
        home_dir().as_deref(),
    )
}

/// Build authority context with explicit paths for deterministic tests.
pub fn build_authority_context_with_paths(
    command: &str,
    source_context: &SourceContext,
    registry: Option<&RegistrySource>,
    package_scope: Option<String>,
    cwd: Option<&Path>,
    home: Option<&Path>,
) -> AuthorityContext {
    let command_intent = redacted_command(command, home);
    let cwd = cwd
        .map(|path| redacted_path(path, home))
        .unwrap_or_else(|| AuthorityDisplayValue {
            category: "unknown".to_string(),
            display: "unknown".to_string(),
        });
    let raw_registry = registry;
    let registry = raw_registry.map(redacted_registry);
    let identity = AuthorityIdentityContext {
        status: "canonical_redacted_identity_v0",
        command_intent_key: digest_report_key(
            "command",
            &redact_report_value_for_home(command, home),
        ),
        cwd_trust_class: cwd.category.clone(),
        registry_key: raw_registry
            .zip(registry.as_ref())
            .and_then(|(raw, redacted)| registry_identity_key(raw, redacted)),
    };

    AuthorityContext {
        command_intent,
        source_context: source_context.clone(),
        runner_context: runner_context(source_context),
        actor_context: actor_context(source_context),
        cwd,
        registry,
        package_scope,
        identity,
        sandbox_boundary:
            "authority context describes ambient process authority; it is not sandboxing",
    }
}

/// Return package scope display for a parsed command.
pub fn package_scope_for_parse(package_spec: &PackageSpecParse) -> Option<String> {
    match package_spec {
        PackageSpecParse::Supported(package_spec) => Some(
            package_spec
                .scope
                .as_ref()
                .map(|_| "scoped")
                .unwrap_or("unscoped")
                .to_string(),
        ),
        PackageSpecParse::Unsupported(_) | PackageSpecParse::Malformed(_) => None,
    }
}

fn runner_context(source_context: &SourceContext) -> AuthorityRunnerContext {
    match source_context {
        SourceContext::ManualTerminal => AuthorityRunnerContext::LocalTerminal,
        SourceContext::Ci => AuthorityRunnerContext::Ci,
        SourceContext::AgentSkill => AuthorityRunnerContext::Agent,
        SourceContext::DocsSnippet | SourceContext::Unknown => AuthorityRunnerContext::Unknown,
    }
}

fn actor_context(source_context: &SourceContext) -> AuthorityActorContext {
    match source_context {
        SourceContext::ManualTerminal | SourceContext::DocsSnippet => {
            AuthorityActorContext::ManualUser
        }
        SourceContext::AgentSkill => AuthorityActorContext::CodingAgent,
        SourceContext::Ci => AuthorityActorContext::Automation,
        SourceContext::Unknown => AuthorityActorContext::Unknown,
    }
}

fn redacted_registry(registry: &RegistrySource) -> AuthorityRegistryContext {
    let category = if registry.url == crate::PUBLIC_NPM_REGISTRY_URL {
        AuthorityRegistryCategory::PublicNpm
    } else if registry.scope.is_some() {
        AuthorityRegistryCategory::ScopedRegistry
    } else {
        AuthorityRegistryCategory::CustomRegistry
    };

    AuthorityRegistryContext {
        category,
        display_url: redact_report_value(&registry.url),
        scope: registry.scope.clone(),
    }
}

fn redacted_command(command: &str, home: Option<&Path>) -> AuthorityDisplayValue {
    AuthorityDisplayValue {
        category: "command_intent".to_string(),
        display: redact_report_value_for_home(command, home),
    }
}

fn redacted_path(path: &Path, home: Option<&Path>) -> AuthorityDisplayValue {
    let display = if let Some(home) = home {
        path.strip_prefix(home)
            .map(|suffix| {
                if suffix.as_os_str().is_empty() {
                    "<home>".to_string()
                } else {
                    format!("<home>/{}", suffix.display())
                }
            })
            .unwrap_or_else(|_| redacted_absolute_path(path))
    } else {
        redacted_absolute_path(path)
    };
    let category = cwd_category(path, home);

    AuthorityDisplayValue { category, display }
}

fn cwd_category(path: &Path, home: Option<&Path>) -> String {
    let temp = std::env::temp_dir();
    if path.starts_with(&temp) || path.starts_with("/tmp") {
        return "temp_directory".to_string();
    }
    if home.is_some_and(|home| path.starts_with(home)) {
        return "home_subtree".to_string();
    }
    if path.is_absolute() {
        return "trusted_project".to_string();
    }

    "unknown".to_string()
}

fn registry_identity_key(
    raw_registry: &RegistrySource,
    redacted_registry: &AuthorityRegistryContext,
) -> Option<String> {
    Some(format!(
        "{:?}:{}:{}",
        redacted_registry.category,
        raw_registry.scope.as_deref().unwrap_or("unscoped"),
        digest_report_key("registry", &redact_report_value(&raw_registry.url))
    ))
}

fn redacted_absolute_path(path: &Path) -> String {
    if path.is_absolute() {
        if path.starts_with(std::env::temp_dir()) || path.starts_with("/tmp") {
            return "<temp>".to_string();
        }
        return "<absolute-path>".to_string();
    }
    path.display().to_string()
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
