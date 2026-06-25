//! Deterministic M2 registry precedence and agreement checks.

use crate::{ClosureDecision, M2Reason, PackageSpec, RegistrySource, PUBLIC_NPM_REGISTRY_URL};

/// Local npm registry configuration supplied by a controlled fixture.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NpmRegistryConfig {
    /// Environment registry override, equivalent to `NPM_CONFIG_REGISTRY`.
    pub env_registry: Option<String>,
    /// Local `.npmrc` contents from the package authority context.
    pub local_npmrc: Option<String>,
}

/// Registry agreement failure between inspection and execution preparation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryPrecedenceMismatch {
    /// Conservative M2 decision for this mismatch.
    pub decision: ClosureDecision,
    /// Stable M2 reason.
    pub reason: M2Reason,
    /// Registry source used during inspection.
    pub inspected: RegistrySource,
    /// Registry source that would be used during execution preparation.
    pub execution: RegistrySource,
}

/// Resolve the deterministic M2 registry source for a package spec.
pub fn resolve_registry_source(
    package_spec: &PackageSpec,
    config: &NpmRegistryConfig,
) -> RegistrySource {
    if let Some(url) = config
        .env_registry
        .as_deref()
        .filter(|url| !url.trim().is_empty())
    {
        return RegistrySource {
            url: normalize_registry_url(url),
            scope: package_spec.scope.clone(),
        };
    }

    let npmrc = config.local_npmrc.as_deref().unwrap_or_default();
    if let Some(scope) = &package_spec.scope {
        if let Some(url) = scoped_registry(npmrc, scope) {
            return RegistrySource {
                url,
                scope: Some(scope.clone()),
            };
        }
    }

    if let Some(url) = unscoped_registry(npmrc) {
        return RegistrySource { url, scope: None };
    }

    RegistrySource {
        url: PUBLIC_NPM_REGISTRY_URL.to_string(),
        scope: None,
    }
}

/// Require inspection and execution preparation to choose the same registry.
pub fn require_registry_agreement(
    inspected: &RegistrySource,
    execution: &RegistrySource,
) -> Result<(), RegistryPrecedenceMismatch> {
    if inspected == execution {
        return Ok(());
    }

    Err(RegistryPrecedenceMismatch {
        decision: ClosureDecision::ExecutionRefused,
        reason: M2Reason::RegistryPrecedenceMismatch,
        inspected: inspected.clone(),
        execution: execution.clone(),
    })
}

/// Normalize registry URLs so equality checks are stable.
pub fn normalize_registry_url(registry_url: &str) -> String {
    let mut trimmed = registry_url.trim().to_string();
    if !trimmed.ends_with('/') {
        trimmed.push('/');
    }
    trimmed
}

fn scoped_registry(npmrc: &str, scope: &str) -> Option<String> {
    let key = format!("@{scope}:registry");
    npmrc_value(npmrc, &key).map(|value| normalize_registry_url(&value))
}

fn unscoped_registry(npmrc: &str) -> Option<String> {
    npmrc_value(npmrc, "registry").map(|value| normalize_registry_url(&value))
}

fn npmrc_value(npmrc: &str, key: &str) -> Option<String> {
    npmrc
        .lines()
        .filter_map(parse_npmrc_line)
        .find_map(|(name, value)| {
            if name == key {
                Some(value.to_string())
            } else {
                None
            }
        })
}

fn parse_npmrc_line(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
        return None;
    }
    let (key, value) = trimmed.split_once('=')?;
    let key = key.trim();
    let value = value.trim();
    if key.is_empty() || value.is_empty() {
        return None;
    }
    Some((key, value))
}

#[cfg(test)]
/// Tests for deterministic registry precedence and agreement checks.
mod tests {
    use super::*;

    #[test]
    /// Verifies public npm is the deterministic fallback.
    fn public_default_registry_is_recorded() {
        let source = resolve_registry_source(&unscoped_spec(), &NpmRegistryConfig::default());

        assert_eq!(source.url, PUBLIC_NPM_REGISTRY_URL);
        assert_eq!(source.scope, None);
    }

    #[test]
    /// Verifies local `.npmrc` selects an unscoped registry.
    fn local_unscoped_registry_is_selected() {
        let source = resolve_registry_source(
            &unscoped_spec(),
            &NpmRegistryConfig {
                env_registry: None,
                local_npmrc: Some("registry=https://fixture.registry.test/npm".to_string()),
            },
        );

        assert_eq!(source.url, "https://fixture.registry.test/npm/");
        assert_eq!(source.scope, None);
    }

    #[test]
    /// Verifies scoped local registry selection from a fixture `.npmrc`.
    fn scoped_registry_is_selected_from_local_npmrc() {
        let source = resolve_registry_source(
            &scoped_spec(),
            &NpmRegistryConfig {
                env_registry: None,
                local_npmrc: Some(
                    "registry=https://fixture.registry.test/default\n@scope:registry=https://scope.registry.test/npm".to_string(),
                ),
            },
        );

        assert_eq!(source.url, "https://scope.registry.test/npm/");
        assert_eq!(source.scope.as_deref(), Some("scope"));
    }

    #[test]
    /// Verifies env registry takes precedence over local `.npmrc`.
    fn env_registry_override_wins() {
        let source = resolve_registry_source(
            &scoped_spec(),
            &NpmRegistryConfig {
                env_registry: Some("https://env.registry.test".to_string()),
                local_npmrc: Some("@scope:registry=https://scope.registry.test".to_string()),
            },
        );

        assert_eq!(source.url, "https://env.registry.test/");
        assert_eq!(source.scope.as_deref(), Some("scope"));
    }

    #[test]
    /// Verifies execution refuses when registry sources disagree.
    fn conflicting_registry_sources_fail_closed() {
        let inspected = RegistrySource {
            url: "https://registry.npmjs.org/".to_string(),
            scope: None,
        };
        let execution = RegistrySource {
            url: "https://fixture.registry.test/".to_string(),
            scope: None,
        };

        let mismatch = require_registry_agreement(&inspected, &execution)
            .expect_err("mismatched registry should fail closed");

        assert_eq!(mismatch.decision, ClosureDecision::ExecutionRefused);
        assert_eq!(mismatch.reason, M2Reason::RegistryPrecedenceMismatch);
        assert_eq!(mismatch.inspected, inspected);
        assert_eq!(mismatch.execution, execution);
    }

    #[test]
    /// Verifies matching registry sources can proceed.
    fn matching_registry_sources_agree() {
        let source = RegistrySource {
            url: "https://registry.npmjs.org/".to_string(),
            scope: None,
        };

        assert_eq!(require_registry_agreement(&source, &source), Ok(()));
    }

    #[test]
    /// Verifies comments and whitespace do not create registry sources.
    fn npmrc_comments_and_blank_lines_are_ignored() {
        let source = resolve_registry_source(
            &unscoped_spec(),
            &NpmRegistryConfig {
                env_registry: None,
                local_npmrc: Some(
                    "\n# comment\n; also comment\n registry = https://fixture.registry.test \n"
                        .to_string(),
                ),
            },
        );

        assert_eq!(source.url, "https://fixture.registry.test/");
    }

    fn unscoped_spec() -> PackageSpec {
        PackageSpec::exact("create-example@1.2.3", "create-example", "1.2.3", None)
    }

    fn scoped_spec() -> PackageSpec {
        PackageSpec::exact(
            "@scope/create-example@1.2.3",
            "@scope/create-example",
            "1.2.3",
            Some("scope".to_string()),
        )
    }
}
