//! Optional package evidence parsed from verified `package.json` bytes.

use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// Optional package evidence read from verified root package metadata.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct PackageOptionalEvidence {
    /// Repository URL or shorthand from package metadata when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// License string from package metadata when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Maintainer-like package people metadata when available.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub maintainers: Vec<PackagePerson>,
    /// Provenance-like package fields represented as compact JSON strings.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub provenance: BTreeMap<String, String>,
}

/// Person-like package metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PackagePerson {
    /// Person name when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Person email when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Extract optional package evidence without failing on malformed values.
pub fn extract_package_optional_evidence(package_json: &Value) -> PackageOptionalEvidence {
    PackageOptionalEvidence {
        repository: repository(package_json),
        license: optional_string(package_json.get("license")),
        maintainers: people_array(package_json.get("maintainers")),
        provenance: provenance_fields(package_json),
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

/// Extract an array of person objects or strings.
fn people_array(value: Option<&Value>) -> Vec<PackagePerson> {
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

/// Extract one person from package object or string forms.
fn person(value: Option<&Value>) -> Option<PackagePerson> {
    match value? {
        Value::String(name) if !name.is_empty() => Some(PackagePerson {
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
            (name.is_some() || email.is_some()).then_some(PackagePerson { name, email })
        }
        _ => None,
    }
}

/// Extract provenance-like package fields as compact JSON.
fn provenance_fields(package_json: &Value) -> BTreeMap<String, String> {
    ["provenance", "attestations", "signatures"]
        .into_iter()
        .filter_map(|field| {
            package_json
                .get(field)
                .filter(|value| matches!(value, Value::Array(_) | Value::Object(_)))
                .and_then(|value| serde_json::to_string(value).ok())
                .map(|value| (field.to_string(), value))
        })
        .collect()
}

#[cfg(test)]
/// Tests for package optional evidence parsing.
mod tests {
    use super::*;

    #[test]
    /// Verifies optional package fields are represented when well formed.
    fn extracts_optional_package_evidence() {
        let evidence = extract_package_optional_evidence(&json(
            r#"{
                "repository": {"url": "https://github.com/example/create-example"},
                "license": "Apache-2.0",
                "maintainers": [{"name": "Alice", "email": "alice@example.test"}],
                "attestations": [{"predicateType": "fixture"}]
            }"#,
        ));

        assert_eq!(
            evidence.repository,
            Some("https://github.com/example/create-example".to_string())
        );
        assert_eq!(evidence.license, Some("Apache-2.0".to_string()));
        assert_eq!(evidence.maintainers[0].name, Some("Alice".to_string()));
        assert!(evidence.provenance.contains_key("attestations"));
    }

    #[test]
    /// Verifies malformed optional package fields are ignored.
    fn ignores_malformed_optional_package_evidence() {
        let evidence = extract_package_optional_evidence(&json(
            r#"{
                "repository": 12,
                "license": [],
                "maintainers": "nobody",
                "provenance": null,
                "attestations": "bad",
                "signatures": 12
            }"#,
        ));

        assert_eq!(evidence, PackageOptionalEvidence::default());
    }

    /// Parse JSON fixture text.
    fn json(input: &str) -> Value {
        serde_json::from_str(input).expect("fixture JSON should parse")
    }
}
