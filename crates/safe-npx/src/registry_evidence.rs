//! Registry metadata evidence extraction for inspect reports.

use crate::{PackageSpec, RegistrySource, ResolvedPackage};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// Resolved package coordinates plus separate registry evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedRegistryPackage {
    /// Exact package coordinates required for tarball download.
    pub resolved_package: ResolvedPackage,
    /// Registry facts tied to the resolved exact version.
    pub registry_evidence: RegistryEvidence,
}

/// Registry evidence tied to one resolved exact package version.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RegistryEvidence {
    /// Registry source selected for metadata resolution.
    pub registry: RegistrySource,
    /// Scope category for the requested package.
    pub package_scope: PackageScopeCategory,
    /// Package name from the resolved exact request.
    pub name: String,
    /// Package version from the resolved exact request.
    pub version: String,
    /// Publish time from registry metadata when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_time: Option<String>,
    /// Maintainers from registry metadata when available.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub maintainers: Vec<RegistryPerson>,
    /// Publisher-like npm user metadata when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<RegistryPerson>,
    /// Repository URL or shorthand from registry metadata when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// License string from registry metadata when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Provenance-like optional fields represented as compact JSON strings.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub provenance: BTreeMap<String, String>,
    /// Dist integrity for the resolved exact version.
    pub dist_integrity: String,
    /// Dist tarball URL for the resolved exact version.
    pub tarball_url: String,
    /// Boundary note preventing registry facts from being treated as tarball facts.
    pub evidence_boundary: &'static str,
}

/// Package scope category for registry evidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageScopeCategory {
    /// Package name includes an npm scope.
    Scoped,
    /// Package name has no npm scope.
    Unscoped,
}

/// Person-like registry metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RegistryPerson {
    /// Person name when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Person email when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Build registry evidence from parsed registry metadata.
pub fn build_registry_evidence(
    package_spec: &PackageSpec,
    registry: &RegistrySource,
    root_metadata: &Value,
    version_metadata: &Value,
    dist: &Value,
    resolved_package: &ResolvedPackage,
) -> RegistryEvidence {
    RegistryEvidence {
        registry: registry.clone(),
        package_scope: if package_spec.scope.is_some() {
            PackageScopeCategory::Scoped
        } else {
            PackageScopeCategory::Unscoped
        },
        name: resolved_package.name.clone(),
        version: resolved_package.version.clone(),
        publish_time: publish_time(root_metadata, &resolved_package.version),
        maintainers: people_array(root_metadata.get("maintainers")),
        publisher: person(version_metadata.get("_npmUser")),
        repository: repository(version_metadata).or_else(|| repository(root_metadata)),
        license: optional_string(version_metadata.get("license"))
            .or_else(|| optional_string(root_metadata.get("license"))),
        provenance: provenance_fields(dist),
        dist_integrity: resolved_package.integrity.clone(),
        tarball_url: resolved_package.tarball_url.clone(),
        evidence_boundary: "registry metadata is not proof of tarball package contents",
    }
}

/// Extract publish time for the resolved version.
fn publish_time(root_metadata: &Value, version: &str) -> Option<String> {
    root_metadata
        .get("time")
        .and_then(|time| time.get(version))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

/// Extract an array of person objects or strings.
fn people_array(value: Option<&Value>) -> Vec<RegistryPerson> {
    value
        .and_then(Value::as_array)
        .map(|people| {
            people
                .iter()
                .filter_map(|entry| person(Some(entry)))
                .collect()
        })
        .unwrap_or_default()
}

/// Extract one person from npm object or string forms.
fn person(value: Option<&Value>) -> Option<RegistryPerson> {
    match value? {
        Value::String(name) if !name.is_empty() => Some(RegistryPerson {
            name: Some(name.clone()),
            email: None,
        }),
        Value::Object(object) => {
            let name = object
                .get("name")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned);
            let email = object
                .get("email")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned);
            if name.is_some() || email.is_some() {
                Some(RegistryPerson { name, email })
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Extract repository metadata from string or object forms.
fn repository(value: &Value) -> Option<String> {
    match value.get("repository")? {
        Value::String(repository) if !repository.is_empty() => Some(repository.clone()),
        Value::Object(object) => object
            .get("url")
            .and_then(Value::as_str)
            .filter(|repository| !repository.is_empty())
            .map(ToOwned::to_owned),
        _ => None,
    }
}

/// Extract an optional string field.
fn optional_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

/// Extract provenance-like dist fields as compact JSON.
fn provenance_fields(dist: &Value) -> BTreeMap<String, String> {
    ["provenance", "attestations", "signatures"]
        .into_iter()
        .filter_map(|field| {
            dist.get(field)
                .and_then(|value| serde_json::to_string(value).ok())
                .map(|value| (format!("dist.{field}"), value))
        })
        .collect()
}

#[cfg(test)]
/// Tests for registry evidence extraction.
mod tests {
    use super::*;
    use crate::RegistrySource;

    #[test]
    /// Verifies public npm-style optional metadata is represented.
    fn extracts_public_registry_optional_metadata() {
        let root = json(
            r#"{
                "time": {"1.2.3": "2026-06-26T07:00:00.000Z"},
                "maintainers": [{"name": "Alice", "email": "alice@example.test"}],
                "repository": {"url": "https://github.com/example/create-example"},
                "license": "Apache-2.0"
            }"#,
        );
        let version = json(
            r#"{
                "_npmUser": {"name": "Publisher", "email": "publisher@example.test"},
                "repository": {"url": "https://github.com/example/version-repo"},
                "license": "MIT"
            }"#,
        );
        let dist = json(
            r#"{
                "integrity": "sha512-fixture",
                "tarball": "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz",
                "signatures": [{"keyid": "fixture"}]
            }"#,
        );
        let evidence = evidence(&root, &version, &dist, package_spec(None));

        assert_eq!(evidence.package_scope, PackageScopeCategory::Unscoped);
        assert_eq!(
            evidence.publish_time,
            Some("2026-06-26T07:00:00.000Z".to_string())
        );
        assert_eq!(evidence.maintainers[0].name, Some("Alice".to_string()));
        assert_eq!(
            evidence.publisher.expect("publisher should parse").name,
            Some("Publisher".to_string())
        );
        assert_eq!(
            evidence.repository,
            Some("https://github.com/example/version-repo".to_string())
        );
        assert_eq!(evidence.license, Some("MIT".to_string()));
        assert!(evidence.provenance.contains_key("dist.signatures"));
        assert_eq!(evidence.dist_integrity, "sha512-fixture");
    }

    #[test]
    /// Verifies missing optional fields do not create placeholder evidence.
    fn missing_optional_metadata_is_absent() {
        let evidence = evidence(&json("{}"), &json("{}"), &dist_json(), package_spec(None));

        assert_eq!(evidence.publish_time, None);
        assert!(evidence.maintainers.is_empty());
        assert_eq!(evidence.publisher, None);
        assert_eq!(evidence.repository, None);
        assert_eq!(evidence.license, None);
        assert!(evidence.provenance.is_empty());
    }

    #[test]
    /// Verifies scoped package metadata records scoped category and registry.
    fn scoped_registry_metadata_records_scope_category() {
        let evidence = evidence(
            &json("{}"),
            &json("{}"),
            &dist_json(),
            package_spec(Some("scope")),
        );

        assert_eq!(evidence.package_scope, PackageScopeCategory::Scoped);
        assert_eq!(evidence.registry.scope, Some("scope".to_string()));
    }

    #[test]
    /// Verifies malformed optional metadata is ignored without failure.
    fn malformed_optional_metadata_is_ignored() {
        let root = json(r#"{"time": [], "maintainers": "nobody", "license": {"bad": true}}"#);
        let version = json(r#"{"_npmUser": [], "repository": 12, "license": []}"#);
        let evidence = evidence(&root, &version, &dist_json(), package_spec(None));

        assert_eq!(evidence.publish_time, None);
        assert!(evidence.maintainers.is_empty());
        assert_eq!(evidence.publisher, None);
        assert_eq!(evidence.repository, None);
        assert_eq!(evidence.license, None);
    }

    /// Build registry evidence from fixture values.
    fn evidence(
        root: &Value,
        version: &Value,
        dist: &Value,
        package_spec: PackageSpec,
    ) -> RegistryEvidence {
        let registry = RegistrySource {
            url: "https://registry.npmjs.org/".to_string(),
            scope: package_spec.scope.clone(),
        };
        let resolved = ResolvedPackage {
            name: package_spec.name.clone(),
            version: package_spec.version.clone(),
            registry: registry.clone(),
            tarball_url: "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz"
                .to_string(),
            integrity: "sha512-fixture".to_string(),
        };

        build_registry_evidence(&package_spec, &registry, root, version, dist, &resolved)
    }

    /// Return a package spec fixture.
    fn package_spec(scope: Option<&str>) -> PackageSpec {
        match scope {
            Some(scope) => PackageSpec::exact(
                "@scope/create-example@1.2.3",
                "@scope/create-example",
                "1.2.3",
                Some(scope.to_string()),
            ),
            None => PackageSpec::exact("create-example@1.2.3", "create-example", "1.2.3", None),
        }
    }

    /// Return a minimal dist object fixture.
    fn dist_json() -> Value {
        json(
            r#"{
                "integrity": "sha512-fixture",
                "tarball": "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz"
            }"#,
        )
    }

    /// Parse JSON fixture text.
    fn json(input: &str) -> Value {
        serde_json::from_str(input).expect("fixture JSON should parse")
    }
}
