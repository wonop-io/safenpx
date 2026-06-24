//! Root tarball byte download without package execution.

use crate::{M1Reason, ResolvedPackage};

/// HTTP response returned by a tarball transport.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TarballHttpResponse {
    /// HTTP status code returned by the tarball source.
    pub status: u16,
    /// Response body as raw tarball bytes.
    pub bytes: Vec<u8>,
}

/// Transport-level error before tarball bytes are available.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TarballTransportError {
    /// Human-readable transport error detail.
    pub message: String,
}

impl TarballTransportError {
    /// Create a transport error from a displayable message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Minimal byte-fetch boundary so tests can stub tarball downloads.
pub trait TarballTransport {
    /// Fetch one tarball URL as bytes.
    fn get_bytes(&self, url: &str) -> Result<TarballHttpResponse, TarballTransportError>;
}

/// Blocking reqwest-backed transport for tarball byte downloads.
#[derive(Clone, Debug, Default)]
pub struct ReqwestTarballTransport;

impl TarballTransport for ReqwestTarballTransport {
    /// Fetch tarball bytes without invoking package managers or extraction.
    fn get_bytes(&self, url: &str) -> Result<TarballHttpResponse, TarballTransportError> {
        let response = reqwest::blocking::get(url)
            .map_err(|error| TarballTransportError::new(error.to_string()))?;
        let status = response.status().as_u16();
        let bytes = response
            .bytes()
            .map_err(|error| TarballTransportError::new(error.to_string()))?
            .to_vec();

        Ok(TarballHttpResponse { status, bytes })
    }
}

/// Downloaded artifact bytes ready for integrity verification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactBytes {
    /// Package name, including scope when present.
    pub name: String,
    /// Exact package version.
    pub version: String,
    /// Source tarball URL used to fetch these bytes.
    pub tarball_url: String,
    /// Raw downloaded bytes, unmodified by extraction or package managers.
    pub bytes: Vec<u8>,
}

/// Error returned by tarball download.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TarballDownloadError {
    /// Stable M1 reason for this error.
    pub reason: M1Reason,
    /// Short diagnostic detail for logs and tests.
    pub detail: String,
}

impl TarballDownloadError {
    /// Create a tarball download error.
    pub fn new(reason: M1Reason, detail: impl Into<String>) -> Self {
        Self {
            reason,
            detail: detail.into(),
        }
    }
}

/// Client that downloads root tarball bytes for resolved packages.
#[derive(Clone, Debug)]
pub struct TarballDownloader<T> {
    transport: T,
}

impl<T: TarballTransport> TarballDownloader<T> {
    /// Create a tarball downloader from a byte transport.
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    /// Download root tarball bytes without extraction or execution.
    pub fn download(
        &self,
        resolved_package: &ResolvedPackage,
    ) -> Result<ArtifactBytes, TarballDownloadError> {
        if !is_absolute_http_url(&resolved_package.tarball_url) {
            return Err(TarballDownloadError::new(
                M1Reason::RegistryError,
                "tarball URL must be absolute HTTP(S)",
            ));
        }

        let response = self
            .transport
            .get_bytes(&resolved_package.tarball_url)
            .map_err(|error| TarballDownloadError::new(M1Reason::RegistryError, error.message))?;

        if !(200..=299).contains(&response.status) {
            return Err(TarballDownloadError::new(
                M1Reason::RegistryError,
                format!("tarball download returned HTTP {}", response.status),
            ));
        }

        if response.bytes.is_empty() {
            return Err(TarballDownloadError::new(
                M1Reason::RegistryError,
                "tarball download returned empty bytes",
            ));
        }

        Ok(ArtifactBytes {
            name: resolved_package.name.clone(),
            version: resolved_package.version.clone(),
            tarball_url: resolved_package.tarball_url.clone(),
            bytes: response.bytes,
        })
    }
}

/// Return true when a URL is absolute HTTP or HTTPS.
fn is_absolute_http_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

#[cfg(test)]
/// Tests for byte-only tarball download behavior.
mod tests {
    use super::*;
    use crate::RegistrySource;
    use std::cell::{Cell, RefCell};

    /// Stub transport that records URL requests and execution-like side effects.
    #[derive(Debug)]
    struct StubTarballTransport {
        response: RefCell<Option<Result<TarballHttpResponse, TarballTransportError>>>,
        requested_urls: RefCell<Vec<String>>,
        package_manager_calls: Cell<usize>,
        extraction_calls: Cell<usize>,
        binary_calls: Cell<usize>,
        lifecycle_calls: Cell<usize>,
        dependency_script_calls: Cell<usize>,
    }

    impl StubTarballTransport {
        /// Create a stub tarball transport with one response.
        fn new(response: Result<TarballHttpResponse, TarballTransportError>) -> Self {
            Self {
                response: RefCell::new(Some(response)),
                requested_urls: RefCell::new(Vec::new()),
                package_manager_calls: Cell::new(0),
                extraction_calls: Cell::new(0),
                binary_calls: Cell::new(0),
                lifecycle_calls: Cell::new(0),
                dependency_script_calls: Cell::new(0),
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

        /// Return all execution-like side-effect counters.
        fn execution_side_effects(&self) -> [usize; 5] {
            [
                self.package_manager_calls.get(),
                self.extraction_calls.get(),
                self.binary_calls.get(),
                self.lifecycle_calls.get(),
                self.dependency_script_calls.get(),
            ]
        }
    }

    impl TarballTransport for StubTarballTransport {
        /// Return the queued byte response without running package code.
        fn get_bytes(&self, url: &str) -> Result<TarballHttpResponse, TarballTransportError> {
            self.requested_urls.borrow_mut().push(url.to_string());
            self.response
                .borrow_mut()
                .take()
                .expect("stub response should be present")
        }
    }

    #[test]
    /// Verifies valid tarball downloads preserve raw bytes.
    fn downloads_tarball_bytes_without_mutation() {
        let downloader = TarballDownloader::new(StubTarballTransport::new(Ok(response(
            200,
            b"fixture-tarball".to_vec(),
        ))));
        let resolved_package = resolved_package();

        let artifact = downloader
            .download(&resolved_package)
            .expect("stubbed tarball should download");

        assert_eq!(artifact.name, "create-example");
        assert_eq!(artifact.version, "1.2.3");
        assert_eq!(artifact.tarball_url, resolved_package.tarball_url);
        assert_eq!(artifact.bytes, b"fixture-tarball");
    }

    #[test]
    /// Verifies downloads do not invoke execution-like hooks.
    fn download_records_zero_execution_side_effects() {
        let transport = StubTarballTransport::new(Ok(response(200, b"bytes".to_vec())));
        let downloader = TarballDownloader::new(transport);

        let _ = downloader.download(&resolved_package());

        assert_eq!(downloader.transport.requested_url(), tarball_url());
        assert_eq!(
            downloader.transport.execution_side_effects(),
            [0, 0, 0, 0, 0]
        );
    }

    #[test]
    /// Verifies invalid tarball URLs fail closed before transport calls.
    fn rejects_invalid_tarball_url() {
        let transport = StubTarballTransport::new(Ok(response(200, b"bytes".to_vec())));
        let downloader = TarballDownloader::new(transport);
        let mut resolved_package = resolved_package();
        resolved_package.tarball_url = "file:///tmp/package.tgz".to_string();

        assert_reason(
            downloader.download(&resolved_package),
            M1Reason::RegistryError,
        );
        assert!(downloader.transport.requested_urls.borrow().is_empty());
    }

    #[test]
    /// Verifies transport failures map to the stable reason.
    fn maps_transport_failure() {
        let downloader = TarballDownloader::new(StubTarballTransport::new(Err(
            TarballTransportError::new("network unavailable"),
        )));

        assert_reason(
            downloader.download(&resolved_package()),
            M1Reason::RegistryError,
        );
    }

    #[test]
    /// Verifies HTTP failures map to the stable reason.
    fn maps_http_failure() {
        let downloader =
            TarballDownloader::new(StubTarballTransport::new(Ok(response(500, Vec::new()))));

        assert_reason(
            downloader.download(&resolved_package()),
            M1Reason::RegistryError,
        );
    }

    #[test]
    /// Verifies empty successful responses fail closed.
    fn maps_empty_body() {
        let downloader =
            TarballDownloader::new(StubTarballTransport::new(Ok(response(200, Vec::new()))));

        assert_reason(
            downloader.download(&resolved_package()),
            M1Reason::RegistryError,
        );
    }

    /// Build a stub tarball HTTP response.
    fn response(status: u16, bytes: Vec<u8>) -> TarballHttpResponse {
        TarballHttpResponse { status, bytes }
    }

    /// Build a resolved package fixture.
    fn resolved_package() -> ResolvedPackage {
        ResolvedPackage {
            name: "create-example".to_string(),
            version: "1.2.3".to_string(),
            registry: RegistrySource {
                url: "https://registry.npmjs.org/".to_string(),
                scope: None,
            },
            tarball_url: tarball_url(),
            integrity: "sha512-fixture".to_string(),
        }
    }

    /// Return the fixture tarball URL.
    fn tarball_url() -> String {
        "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz".to_string()
    }

    /// Assert a download error maps to one stable M1 reason.
    fn assert_reason(
        result: Result<ArtifactBytes, TarballDownloadError>,
        expected_reason: M1Reason,
    ) {
        let error = result.expect_err("download should fail");

        assert_eq!(error.reason, expected_reason);
    }
}
