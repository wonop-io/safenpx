//! Public npm registry metadata resolution for exact-version package specs.

use crate::{normalize_registry_url, M1Reason, PackageSpec, RegistrySource, ResolvedPackage};
use serde_json::Value;

/// Public npm registry base URL used by M1.
pub const PUBLIC_NPM_REGISTRY_URL: &str = "https://registry.npmjs.org/";

/// HTTP response returned by a registry transport.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryHttpResponse {
    /// HTTP status code returned by the registry.
    pub status: u16,
    /// Response body as UTF-8 text.
    pub body: String,
}

/// Transport-level error before registry payload interpretation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryTransportError {
    /// Human-readable transport error detail.
    pub message: String,
}

impl RegistryTransportError {
    /// Create a transport error from a displayable message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Minimal HTTP boundary so tests can stub registry behavior.
pub trait RegistryTransport {
    /// Fetch one registry metadata URL.
    fn get(&self, url: &str) -> Result<RegistryHttpResponse, RegistryTransportError>;
}

/// Blocking reqwest-backed transport for the public npm registry.
#[derive(Clone, Debug, Default)]
pub struct ReqwestRegistryTransport;

impl RegistryTransport for ReqwestRegistryTransport {
    /// Fetch one metadata URL without executing package code.
    fn get(&self, url: &str) -> Result<RegistryHttpResponse, RegistryTransportError> {
        let response = reqwest::blocking::get(url)
            .map_err(|error| RegistryTransportError::new(error.to_string()))?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .map_err(|error| RegistryTransportError::new(error.to_string()))?;

        Ok(RegistryHttpResponse { status, body })
    }
}

/// Error returned by metadata resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryResolutionError {
    /// Stable M1 reason for this error.
    pub reason: M1Reason,
    /// Short diagnostic detail for logs and tests.
    pub detail: String,
}

impl RegistryResolutionError {
    /// Create a registry resolution error.
    pub fn new(reason: M1Reason, detail: impl Into<String>) -> Self {
        Self {
            reason,
            detail: detail.into(),
        }
    }
}

/// Client that resolves exact-version package specs through registry metadata.
#[derive(Clone, Debug)]
pub struct NpmMetadataClient<T> {
    registry: RegistrySource,
    transport: T,
}

impl<T: RegistryTransport> NpmMetadataClient<T> {
    /// Create a client for the public npm registry.
    pub fn public(transport: T) -> Self {
        Self::new(crate::PUBLIC_NPM_REGISTRY_URL, transport)
    }

    /// Create a client for a registry base URL.
    pub fn new(registry_url: impl Into<String>, transport: T) -> Self {
        Self {
            registry: RegistrySource {
                url: normalize_registry_url(&registry_url.into()),
                scope: None,
            },
            transport,
        }
    }

    /// Create a client from an already selected registry source.
    pub fn from_registry_source(registry: &RegistrySource, transport: T) -> Self {
        Self {
            registry: registry.clone(),
            transport,
        }
    }

    /// Build the metadata URL used for a package spec.
    pub fn metadata_url(&self, package_spec: &PackageSpec) -> String {
        format!(
            "{}{}",
            self.registry.url,
            encode_package_name(&package_spec.name)
        )
    }

    /// Resolve exact-version metadata into stable M1 package coordinates.
    pub fn resolve_exact(
        &self,
        package_spec: &PackageSpec,
    ) -> Result<ResolvedPackage, RegistryResolutionError> {
        let metadata_url = self.metadata_url(package_spec);
        let response = self.transport.get(&metadata_url).map_err(|error| {
            RegistryResolutionError::new(M1Reason::RegistryError, error.message)
        })?;

        match response.status {
            200..=299 => resolve_from_body(package_spec, &self.registry, &response.body),
            404 => Err(RegistryResolutionError::new(
                M1Reason::MissingPackage,
                "package metadata was not found",
            )),
            status => Err(RegistryResolutionError::new(
                M1Reason::RegistryError,
                format!("registry returned HTTP {status}"),
            )),
        }
    }
}

/// Percent-encode the npm package name path segment.
fn encode_package_name(name: &str) -> String {
    name.bytes()
        .flat_map(|byte| match byte {
            b'@' => "%40".bytes().collect::<Vec<_>>(),
            b'/' => "%2F".bytes().collect::<Vec<_>>(),
            _ => vec![byte],
        })
        .map(char::from)
        .collect()
}

/// Resolve the requested exact version from a registry JSON body.
fn resolve_from_body(
    package_spec: &PackageSpec,
    registry: &RegistrySource,
    body: &str,
) -> Result<ResolvedPackage, RegistryResolutionError> {
    let metadata = serde_json::from_str::<Value>(body).map_err(|error| {
        RegistryResolutionError::new(M1Reason::RegistryError, error.to_string())
    })?;
    let versions = metadata
        .get("versions")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            RegistryResolutionError::new(M1Reason::RegistryError, "missing versions object")
        })?;
    let version_metadata = versions.get(&package_spec.version).ok_or_else(|| {
        RegistryResolutionError::new(M1Reason::MissingVersion, "requested version was not found")
    })?;
    let version = required_string(version_metadata, "version")?;
    if version != package_spec.version {
        return Err(RegistryResolutionError::new(
            M1Reason::RegistryError,
            "selected version metadata did not match requested version",
        ));
    }

    let dist = version_metadata
        .get("dist")
        .ok_or_else(|| RegistryResolutionError::new(M1Reason::RegistryError, "missing dist"))?;
    let tarball_url = required_string(dist, "tarball")?.to_string();
    let integrity = required_string(dist, "integrity")?.to_string();

    Ok(ResolvedPackage {
        name: package_spec.name.clone(),
        version: version.to_string(),
        registry: registry.clone(),
        tarball_url,
        integrity,
    })
}

/// Read a required string field from registry metadata.
fn required_string<'a>(value: &'a Value, field: &str) -> Result<&'a str, RegistryResolutionError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|field_value| !field_value.is_empty())
        .ok_or_else(|| {
            RegistryResolutionError::new(
                M1Reason::RegistryError,
                format!("missing or invalid {field}"),
            )
        })
}

#[cfg(test)]
/// Tests for npm metadata URL construction and error mapping.
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// Stub registry transport that records URLs and returns queued responses.
    #[derive(Debug)]
    struct StubTransport {
        responses: RefCell<Vec<Result<RegistryHttpResponse, RegistryTransportError>>>,
        requested_urls: RefCell<Vec<String>>,
    }

    impl StubTransport {
        /// Create a stub transport with queued responses.
        fn new(responses: Vec<Result<RegistryHttpResponse, RegistryTransportError>>) -> Self {
            Self {
                responses: RefCell::new(responses),
                requested_urls: RefCell::new(Vec::new()),
            }
        }

        /// Return the first requested URL.
        fn requested_url(&self) -> String {
            self.requested_urls
                .borrow()
                .first()
                .expect("stub should record a request")
                .clone()
        }
    }

    impl RegistryTransport for StubTransport {
        /// Return the next queued response without touching the network.
        fn get(&self, url: &str) -> Result<RegistryHttpResponse, RegistryTransportError> {
            self.requested_urls.borrow_mut().push(url.to_string());
            self.responses.borrow_mut().remove(0)
        }
    }

    #[test]
    /// Verifies unscoped package metadata URLs.
    fn builds_unscoped_metadata_url() {
        let client = NpmMetadataClient::public(StubTransport::new(Vec::new()));
        let package_spec =
            PackageSpec::exact("create-example@1.2.3", "create-example", "1.2.3", None);

        assert_eq!(
            client.metadata_url(&package_spec),
            "https://registry.npmjs.org/create-example"
        );
    }

    #[test]
    /// Verifies scoped package metadata URL encoding.
    fn builds_scoped_metadata_url() {
        let client = NpmMetadataClient::public(StubTransport::new(Vec::new()));
        let package_spec = PackageSpec::exact(
            "@scope/create-example@1.2.3",
            "@scope/create-example",
            "1.2.3",
            Some("scope".to_string()),
        );

        assert_eq!(
            client.metadata_url(&package_spec),
            "https://registry.npmjs.org/%40scope%2Fcreate-example"
        );
    }

    #[test]
    /// Verifies exact-version metadata is extracted from a stubbed response.
    fn resolves_exact_version_metadata() {
        let package_spec =
            PackageSpec::exact("create-example@1.2.3", "create-example", "1.2.3", None);
        let transport = StubTransport::new(vec![Ok(response(200, successful_body()))]);
        let client = NpmMetadataClient::public(transport);

        let resolved = client
            .resolve_exact(&package_spec)
            .expect("stubbed metadata should resolve");

        assert_eq!(resolved.name, "create-example");
        assert_eq!(resolved.version, "1.2.3");
        assert_eq!(resolved.registry.url, PUBLIC_NPM_REGISTRY_URL);
        assert_eq!(resolved.registry.scope, None);
        assert_eq!(
            resolved.tarball_url,
            "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz"
        );
        assert_eq!(resolved.integrity, "sha512-fixture");
    }

    #[test]
    /// Verifies resolved metadata preserves the selected scoped registry source.
    fn resolves_with_selected_registry_source() {
        let package_spec = PackageSpec::exact(
            "@scope/create-example@1.2.3",
            "@scope/create-example",
            "1.2.3",
            Some("scope".to_string()),
        );
        let registry = RegistrySource {
            url: "https://scope.registry.test/npm/".to_string(),
            scope: Some("scope".to_string()),
        };
        let transport = StubTransport::new(vec![Ok(response(200, scoped_successful_body()))]);
        let client = NpmMetadataClient::from_registry_source(&registry, transport);

        let resolved = client
            .resolve_exact(&package_spec)
            .expect("stubbed scoped metadata should resolve");

        assert_eq!(resolved.registry, registry);
        assert_eq!(
            resolved.tarball_url,
            "https://scope.registry.test/npm/@scope/create-example/-/create-example-1.2.3.tgz"
        );
    }

    #[test]
    /// Verifies missing package responses use the stable reason.
    fn maps_missing_package() {
        let package_spec =
            PackageSpec::exact("create-example@1.2.3", "create-example", "1.2.3", None);
        let client = NpmMetadataClient::public(StubTransport::new(vec![Ok(response(404, "{}"))]));

        assert_reason(
            client.resolve_exact(&package_spec),
            M1Reason::MissingPackage,
        );
    }

    #[test]
    /// Verifies missing exact versions use the stable reason.
    fn maps_missing_version() {
        let package_spec =
            PackageSpec::exact("create-example@9.9.9", "create-example", "9.9.9", None);
        let client = NpmMetadataClient::public(StubTransport::new(vec![Ok(response(
            200,
            successful_body(),
        ))]));

        assert_reason(
            client.resolve_exact(&package_spec),
            M1Reason::MissingVersion,
        );
    }

    #[test]
    /// Verifies transport failures use the registry error reason.
    fn maps_transport_failure() {
        let package_spec =
            PackageSpec::exact("create-example@1.2.3", "create-example", "1.2.3", None);
        let client = NpmMetadataClient::public(StubTransport::new(vec![Err(
            RegistryTransportError::new("network unavailable"),
        )]));

        assert_reason(client.resolve_exact(&package_spec), M1Reason::RegistryError);
    }

    #[test]
    /// Verifies invalid JSON uses the registry error reason.
    fn maps_invalid_json() {
        let package_spec =
            PackageSpec::exact("create-example@1.2.3", "create-example", "1.2.3", None);
        let client =
            NpmMetadataClient::public(StubTransport::new(vec![Ok(response(200, "not json"))]));

        assert_reason(client.resolve_exact(&package_spec), M1Reason::RegistryError);
    }

    #[test]
    /// Verifies invalid dist payloads use the registry error reason.
    fn maps_invalid_dist_payload() {
        let package_spec =
            PackageSpec::exact("create-example@1.2.3", "create-example", "1.2.3", None);
        let client = NpmMetadataClient::public(StubTransport::new(vec![Ok(response(
            200,
            r#"{"versions":{"1.2.3":{"version":"1.2.3","dist":{}}}}"#,
        ))]));

        assert_reason(client.resolve_exact(&package_spec), M1Reason::RegistryError);
    }

    #[test]
    /// Verifies non-404 HTTP failures use the registry error reason.
    fn maps_non_404_http_failure() {
        let package_spec =
            PackageSpec::exact("create-example@1.2.3", "create-example", "1.2.3", None);
        let client = NpmMetadataClient::public(StubTransport::new(vec![Ok(response(500, "{}"))]));

        assert_reason(client.resolve_exact(&package_spec), M1Reason::RegistryError);
    }

    #[test]
    /// Verifies the client requests the encoded metadata URL through the transport.
    fn requests_encoded_metadata_url() {
        let package_spec = PackageSpec::exact(
            "@scope/create-example@1.2.3",
            "@scope/create-example",
            "1.2.3",
            Some("scope".to_string()),
        );
        let transport = StubTransport::new(vec![Ok(response(200, successful_body()))]);
        let client = NpmMetadataClient::public(transport);

        let _ = client.resolve_exact(&package_spec);

        assert_eq!(
            client.transport.requested_url(),
            "https://registry.npmjs.org/%40scope%2Fcreate-example"
        );
    }

    /// Build a stub HTTP response.
    fn response(status: u16, body: &str) -> RegistryHttpResponse {
        RegistryHttpResponse {
            status,
            body: body.to_string(),
        }
    }

    /// Return a minimal successful npm metadata payload.
    fn successful_body() -> &'static str {
        r#"{
            "name": "create-example",
            "versions": {
                "1.2.3": {
                    "name": "create-example",
                    "version": "1.2.3",
                    "dist": {
                        "tarball": "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz",
                        "integrity": "sha512-fixture"
                    }
                }
            }
        }"#
    }

    /// Assert a resolution error maps to one stable M1 reason.
    fn assert_reason(
        result: Result<ResolvedPackage, RegistryResolutionError>,
        expected_reason: M1Reason,
    ) {
        let error = result.expect_err("resolution should fail");

        assert_eq!(error.reason, expected_reason);
    }

    /// Build a successful scoped registry metadata body.
    fn scoped_successful_body() -> &'static str {
        r#"{
            "versions": {
                "1.2.3": {
                    "version": "1.2.3",
                    "dist": {
                        "tarball": "https://scope.registry.test/npm/@scope/create-example/-/create-example-1.2.3.tgz",
                        "integrity": "sha512-fixture"
                    }
                }
            }
        }"#
    }
}
