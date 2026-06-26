//! End-to-end root npm artifact resolution for M1.

use crate::{
    verify_artifact_integrity, ArtifactIdentity, ArtifactVerificationError, Decision, M1Reason,
    NpmMetadataClient, PackageSpec, RegistryEvidence, RegistryResolutionError, RegistryTransport,
    ResolvedPackage, TarballDownloadError, TarballDownloader, TarballTransport,
};

/// Verified root artifact resolution result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedRootArtifact {
    /// Exact package metadata resolved from npm.
    pub resolved_package: ResolvedPackage,
    /// Verified identity for the exact downloaded root artifact bytes.
    pub artifact_identity: ArtifactIdentity,
    /// Registry facts tied to the resolved exact version.
    pub registry_evidence: RegistryEvidence,
    /// Exact tarball bytes verified by `artifact_identity`.
    pub artifact_bytes: Vec<u8>,
}

/// Error returned by end-to-end root artifact resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootArtifactResolutionError {
    /// Policy decision when the failure is a deny decision.
    pub decision: Option<Decision>,
    /// Stable M1 reason for the failure.
    pub reason: M1Reason,
    /// Short diagnostic detail for logs and tests.
    pub detail: String,
}

impl RootArtifactResolutionError {
    /// Create a resolver error without a policy decision.
    pub fn inspection_error(reason: M1Reason, detail: impl Into<String>) -> Self {
        Self {
            decision: None,
            reason,
            detail: detail.into(),
        }
    }

    /// Create a resolver error with an explicit policy decision.
    pub fn with_decision(decision: Decision, reason: M1Reason, detail: impl Into<String>) -> Self {
        Self {
            decision: Some(decision),
            reason,
            detail: detail.into(),
        }
    }
}

impl From<RegistryResolutionError> for RootArtifactResolutionError {
    /// Preserve stable registry failure reasons.
    fn from(error: RegistryResolutionError) -> Self {
        Self::inspection_error(error.reason, error.detail)
    }
}

impl From<TarballDownloadError> for RootArtifactResolutionError {
    /// Preserve stable download failure reasons.
    fn from(error: TarballDownloadError) -> Self {
        Self::inspection_error(error.reason, error.detail)
    }
}

impl From<ArtifactVerificationError> for RootArtifactResolutionError {
    /// Preserve integrity mismatch denial behavior.
    fn from(error: ArtifactVerificationError) -> Self {
        Self::with_decision(error.decision, error.reason, error.detail)
    }
}

/// Resolver that composes metadata, download, and integrity verification.
#[derive(Clone, Debug)]
pub struct RootArtifactResolver<M, D> {
    metadata_client: NpmMetadataClient<M>,
    tarball_downloader: TarballDownloader<D>,
}

impl<M: RegistryTransport, D: TarballTransport> RootArtifactResolver<M, D> {
    /// Create a root artifact resolver from testable resolver components.
    pub fn new(
        metadata_client: NpmMetadataClient<M>,
        tarball_downloader: TarballDownloader<D>,
    ) -> Self {
        Self {
            metadata_client,
            tarball_downloader,
        }
    }

    /// Resolve, download, and verify the root artifact for a supported spec.
    pub fn resolve(
        &self,
        package_spec: &PackageSpec,
    ) -> Result<VerifiedRootArtifact, RootArtifactResolutionError> {
        let resolved_registry_package = self.metadata_client.resolve_exact(package_spec)?;
        let resolved_package = resolved_registry_package.resolved_package;
        let artifact_bytes = self.tarball_downloader.download(&resolved_package)?;
        let artifact_identity =
            verify_artifact_integrity(&artifact_bytes, &resolved_package.integrity)?;

        Ok(VerifiedRootArtifact {
            resolved_package,
            artifact_identity,
            registry_evidence: resolved_registry_package.registry_evidence,
            artifact_bytes: artifact_bytes.bytes,
        })
    }
}

#[cfg(test)]
/// Tests for end-to-end root artifact resolution.
mod tests {
    use super::*;
    use crate::{
        RegistryHttpResponse, RegistryTransportError, TarballHttpResponse, TarballTransportError,
    };
    use base64::prelude::{Engine as _, BASE64_STANDARD};
    use sha2::{Digest, Sha512};
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    /// Shared counters for execution-like side effects.
    #[derive(Default)]
    struct ExecutionCounters {
        package_manager_calls: Cell<usize>,
        extraction_calls: Cell<usize>,
        binary_calls: Cell<usize>,
        lifecycle_calls: Cell<usize>,
        dependency_script_calls: Cell<usize>,
    }

    impl ExecutionCounters {
        /// Return all execution-like side-effect counters.
        fn snapshot(&self) -> [usize; 5] {
            [
                self.package_manager_calls.get(),
                self.extraction_calls.get(),
                self.binary_calls.get(),
                self.lifecycle_calls.get(),
                self.dependency_script_calls.get(),
            ]
        }
    }

    /// Stub metadata transport with queued local responses.
    #[derive(Clone)]
    struct StubRegistryTransport {
        responses: Rc<RefCell<Vec<Result<RegistryHttpResponse, RegistryTransportError>>>>,
    }

    impl StubRegistryTransport {
        /// Create a stub metadata transport.
        fn new(responses: Vec<Result<RegistryHttpResponse, RegistryTransportError>>) -> Self {
            Self {
                responses: Rc::new(RefCell::new(responses)),
            }
        }
    }

    impl RegistryTransport for StubRegistryTransport {
        /// Return the next queued metadata response.
        fn get(&self, _url: &str) -> Result<RegistryHttpResponse, RegistryTransportError> {
            self.responses.borrow_mut().remove(0)
        }
    }

    /// Stub tarball transport with queued local responses and side-effect counters.
    #[derive(Clone)]
    struct StubTarballTransport {
        responses: Rc<RefCell<Vec<Result<TarballHttpResponse, TarballTransportError>>>>,
        counters: Rc<ExecutionCounters>,
    }

    impl StubTarballTransport {
        /// Create a stub tarball transport.
        fn new(
            responses: Vec<Result<TarballHttpResponse, TarballTransportError>>,
            counters: Rc<ExecutionCounters>,
        ) -> Self {
            Self {
                responses: Rc::new(RefCell::new(responses)),
                counters,
            }
        }
    }

    impl TarballTransport for StubTarballTransport {
        /// Return the next queued byte response without executing package code.
        fn get_bytes(&self, _url: &str) -> Result<TarballHttpResponse, TarballTransportError> {
            let _ = self.counters.snapshot();
            self.responses.borrow_mut().remove(0)
        }
    }

    #[test]
    /// Verifies a valid package resolves to metadata and digest identity.
    fn resolves_verified_root_artifact() {
        let bytes = b"fixture-tarball".to_vec();
        let integrity = integrity_for(&bytes);
        let counters = Rc::new(ExecutionCounters::default());
        let resolver = resolver_with(
            vec![Ok(registry_response(200, &metadata_body(&integrity)))],
            vec![Ok(tarball_response(200, bytes.clone()))],
            Rc::clone(&counters),
        );

        let verified = resolver
            .resolve(&package_spec())
            .expect("root artifact should resolve");

        assert_eq!(verified.resolved_package.name, "create-example");
        assert_eq!(verified.resolved_package.version, "1.2.3");
        assert_eq!(
            verified.resolved_package.registry.url,
            "https://registry.npmjs.org/"
        );
        assert_eq!(verified.resolved_package.tarball_url, tarball_url());
        assert_eq!(verified.resolved_package.integrity, integrity);
        assert_eq!(verified.artifact_identity.name, "create-example");
        assert_eq!(verified.artifact_identity.version, "1.2.3");
        assert_eq!(verified.artifact_identity.integrity, integrity);
        assert_eq!(verified.artifact_identity.digest_algorithm, "sha512");
        assert_eq!(verified.artifact_identity.digest, hex_sha512(&bytes));
        assert_eq!(verified.registry_evidence.name, "create-example");
        assert_eq!(verified.registry_evidence.dist_integrity, integrity);
        assert_eq!(verified.registry_evidence.tarball_url, tarball_url());
        assert_eq!(verified.artifact_bytes, bytes);
        assert_eq!(counters.snapshot(), [0, 0, 0, 0, 0]);
    }

    #[test]
    /// Verifies missing packages preserve the stable reason.
    fn maps_missing_package() {
        let resolver = resolver_with(
            vec![Ok(registry_response(404, "{}"))],
            Vec::new(),
            Rc::new(ExecutionCounters::default()),
        );

        assert_failure_reason(
            resolver.resolve(&package_spec()),
            M1Reason::MissingPackage,
            None,
        );
    }

    #[test]
    /// Verifies missing versions preserve the stable reason.
    fn maps_missing_version() {
        let resolver = resolver_with(
            vec![Ok(registry_response(
                200,
                r#"{"versions":{"9.9.9":{"version":"9.9.9","dist":{"tarball":"https://registry.npmjs.org/create-example/-/create-example-9.9.9.tgz","integrity":"sha512-fixture"}}}}"#,
            ))],
            Vec::new(),
            Rc::new(ExecutionCounters::default()),
        );

        assert_failure_reason(
            resolver.resolve(&package_spec()),
            M1Reason::MissingVersion,
            None,
        );
    }

    #[test]
    /// Verifies registry failures preserve the stable reason.
    fn maps_registry_failure() {
        let resolver = resolver_with(
            vec![Err(RegistryTransportError::new("registry unavailable"))],
            Vec::new(),
            Rc::new(ExecutionCounters::default()),
        );

        assert_failure_reason(
            resolver.resolve(&package_spec()),
            M1Reason::RegistryError,
            None,
        );
    }

    #[test]
    /// Verifies download failures preserve the stable reason.
    fn maps_download_failure() {
        let bytes = b"fixture-tarball".to_vec();
        let resolver = resolver_with(
            vec![Ok(registry_response(
                200,
                &metadata_body(&integrity_for(&bytes)),
            ))],
            vec![Err(TarballTransportError::new("download unavailable"))],
            Rc::new(ExecutionCounters::default()),
        );

        assert_failure_reason(
            resolver.resolve(&package_spec()),
            M1Reason::RegistryError,
            None,
        );
    }

    #[test]
    /// Verifies integrity mismatch returns deny and integrity_mismatch.
    fn maps_integrity_mismatch_to_deny() {
        let counters = Rc::new(ExecutionCounters::default());
        let resolver = resolver_with(
            vec![Ok(registry_response(
                200,
                &metadata_body(&integrity_for(b"different-bytes")),
            ))],
            vec![Ok(tarball_response(200, b"fixture-tarball".to_vec()))],
            Rc::clone(&counters),
        );

        assert_failure_reason(
            resolver.resolve(&package_spec()),
            M1Reason::IntegrityMismatch,
            Some(Decision::Deny),
        );
        assert_eq!(counters.snapshot(), [0, 0, 0, 0, 0]);
    }

    /// Build a resolver from local stub responses.
    fn resolver_with(
        registry_responses: Vec<Result<RegistryHttpResponse, RegistryTransportError>>,
        tarball_responses: Vec<Result<TarballHttpResponse, TarballTransportError>>,
        counters: Rc<ExecutionCounters>,
    ) -> RootArtifactResolver<StubRegistryTransport, StubTarballTransport> {
        RootArtifactResolver::new(
            NpmMetadataClient::public(StubRegistryTransport::new(registry_responses)),
            TarballDownloader::new(StubTarballTransport::new(tarball_responses, counters)),
        )
    }

    /// Build the supported package spec fixture.
    fn package_spec() -> PackageSpec {
        PackageSpec::exact("create-example@1.2.3", "create-example", "1.2.3", None)
    }

    /// Build a registry HTTP response.
    fn registry_response(status: u16, body: &str) -> RegistryHttpResponse {
        RegistryHttpResponse {
            status,
            body: body.to_string(),
        }
    }

    /// Build a tarball HTTP response.
    fn tarball_response(status: u16, bytes: Vec<u8>) -> TarballHttpResponse {
        TarballHttpResponse { status, bytes }
    }

    /// Build npm metadata JSON with one exact version.
    fn metadata_body(integrity: &str) -> String {
        format!(
            r#"{{
                "name": "create-example",
                "versions": {{
                    "1.2.3": {{
                        "name": "create-example",
                        "version": "1.2.3",
                        "dist": {{
                            "tarball": "{}",
                            "integrity": "{}"
                        }}
                    }}
                }}
            }}"#,
            tarball_url(),
            integrity
        )
    }

    /// Return the fixture tarball URL.
    fn tarball_url() -> String {
        "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz".to_string()
    }

    /// Return npm sha512 integrity metadata for bytes.
    fn integrity_for(bytes: &[u8]) -> String {
        format!("sha512-{}", BASE64_STANDARD.encode(Sha512::digest(bytes)))
    }

    /// Return lowercase hexadecimal SHA-512 digest for bytes.
    fn hex_sha512(bytes: &[u8]) -> String {
        Sha512::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<Vec<_>>()
            .join("")
    }

    /// Assert one resolver failure reason and optional decision.
    fn assert_failure_reason(
        result: Result<VerifiedRootArtifact, RootArtifactResolutionError>,
        expected_reason: M1Reason,
        expected_decision: Option<Decision>,
    ) {
        let error = result.expect_err("resolution should fail");

        assert_eq!(error.reason, expected_reason);
        assert_eq!(error.decision, expected_decision);
    }
}
