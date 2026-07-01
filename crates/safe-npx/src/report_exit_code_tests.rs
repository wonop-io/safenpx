//! Tests for the M4 report exit-code contract.

use crate::{
    build_m2_execution_refusal_report, build_report_with_resolver, exit_code_for_report, Cli,
    ClosureCommandIdentity, M2Reason, NpmMetadataClient, RegistryHttpResponse, RegistryTransport,
    RegistryTransportError, RootArtifactResolver, TarballDownloader, TarballHttpResponse,
    TarballTransport, TarballTransportError, M2_EXECUTION_REFUSED_EXIT_CODE,
    M4_ASK_REQUIRED_EXIT_CODE, M4_DELEGATED_EXECUTION_FAILED_EXIT_CODE, M4_DENIED_EXIT_CODE,
    M4_INSPECTION_ERROR_EXIT_CODE, M4_SUCCESS_EXIT_CODE, M4_UNSUPPORTED_EXIT_CODE,
};
use base64::prelude::{Engine as _, BASE64_STANDARD};
use sha2::{Digest, Sha512};
use std::cell::RefCell;

#[derive(Debug)]
struct StubRegistryTransport {
    responses: RefCell<Vec<Result<RegistryHttpResponse, RegistryTransportError>>>,
}

impl StubRegistryTransport {
    fn new(responses: Vec<Result<RegistryHttpResponse, RegistryTransportError>>) -> Self {
        Self {
            responses: RefCell::new(responses),
        }
    }
}

impl RegistryTransport for StubRegistryTransport {
    fn get(&self, _url: &str) -> Result<RegistryHttpResponse, RegistryTransportError> {
        self.responses.borrow_mut().remove(0)
    }
}

#[derive(Debug)]
struct StubTarballTransport {
    responses: RefCell<Vec<Result<TarballHttpResponse, TarballTransportError>>>,
}

impl StubTarballTransport {
    fn new(responses: Vec<Result<TarballHttpResponse, TarballTransportError>>) -> Self {
        Self {
            responses: RefCell::new(responses),
        }
    }
}

impl TarballTransport for StubTarballTransport {
    fn get_bytes(&self, _url: &str) -> Result<TarballHttpResponse, TarballTransportError> {
        self.responses.borrow_mut().remove(0)
    }
}

#[test]
fn m4_exit_code_contract_covers_current_decisions() {
    let allow_cli = Cli::parse_from(["safe-npx", "--decision", "allow", "create-example@1.2.3"]);
    let allow_report = build_report_with_resolver(&allow_cli, &verified_resolver(b"fixture"));
    assert_eq!(exit_code_for_report(&allow_report), M4_SUCCESS_EXIT_CODE);

    let deny_cli = Cli::parse_from(["safe-npx", "--decision", "deny", "create-example@1.2.3"]);
    let deny_report = build_report_with_resolver(&deny_cli, &verified_resolver(b"fixture"));
    assert_eq!(exit_code_for_report(&deny_report), M4_DENIED_EXIT_CODE);

    let ask_cli = Cli::parse_from(["safe-npx", "--source-context", "ci", "create-example@1.2.3"]);
    let ask_report = build_report_with_resolver(&ask_cli, &verified_resolver(b"fixture"));
    assert_eq!(exit_code_for_report(&ask_report), M4_ASK_REQUIRED_EXIT_CODE);

    let unsupported_cli = Cli::parse_from(["safe-npx", "create-example@next"]);
    let unsupported_report = build_report_with_resolver(
        &unsupported_cli,
        &RootArtifactResolver::new(
            NpmMetadataClient::public(StubRegistryTransport::new(Vec::new())),
            TarballDownloader::new(StubTarballTransport::new(Vec::new())),
        ),
    );
    assert_eq!(
        exit_code_for_report(&unsupported_report),
        M4_UNSUPPORTED_EXIT_CODE
    );

    let mismatch_cli = Cli::parse_from(["safe-npx", "create-example@1.2.3"]);
    let mismatch_report = build_report_with_resolver(
        &mismatch_cli,
        &resolver_with(
            metadata_body(&integrity_for(b"different-bytes")),
            b"fixture".to_vec(),
        ),
    );
    assert_eq!(exit_code_for_report(&mismatch_report), M4_DENIED_EXIT_CODE);

    let registry_failure_report = build_report_with_resolver(
        &mismatch_cli,
        &RootArtifactResolver::new(
            NpmMetadataClient::public(StubRegistryTransport::new(vec![Ok(RegistryHttpResponse {
                status: 500,
                body: "{}".to_string(),
            })])),
            TarballDownloader::new(StubTarballTransport::new(Vec::new())),
        ),
    );
    assert_eq!(
        exit_code_for_report(&registry_failure_report),
        M4_INSPECTION_ERROR_EXIT_CODE
    );

    let execution_refusal =
        build_m2_execution_refusal_report(command_identity(), vec![M2Reason::UnsupportedClosure]);
    assert_eq!(execution_refusal.exit_code, M2_EXECUTION_REFUSED_EXIT_CODE);
    assert_eq!(M4_DELEGATED_EXECUTION_FAILED_EXIT_CODE, 15);
}

fn command_identity() -> ClosureCommandIdentity {
    ClosureCommandIdentity {
        requested: "create-example@1.2.3".to_string(),
        forwarded_args: vec!["--template".to_string(), "react".to_string()],
    }
}

fn verified_resolver(
    bytes: &[u8],
) -> RootArtifactResolver<StubRegistryTransport, StubTarballTransport> {
    resolver_with(metadata_body(&integrity_for(bytes)), bytes.to_vec())
}

fn resolver_with(
    metadata_body: String,
    tarball_bytes: Vec<u8>,
) -> RootArtifactResolver<StubRegistryTransport, StubTarballTransport> {
    RootArtifactResolver::new(
        NpmMetadataClient::public(StubRegistryTransport::new(vec![Ok(RegistryHttpResponse {
            status: 200,
            body: metadata_body,
        })])),
        TarballDownloader::new(StubTarballTransport::new(vec![Ok(TarballHttpResponse {
            status: 200,
            bytes: tarball_bytes,
        })])),
    )
}

fn metadata_body(integrity: &str) -> String {
    format!(
        r#"{{
            "versions": {{
                "1.2.3": {{
                    "version": "1.2.3",
                    "dist": {{
                        "tarball": "https://registry.npmjs.org/create-example/-/create-example-1.2.3.tgz",
                        "integrity": "{}"
                    }}
                }}
            }}
        }}"#,
        integrity
    )
}

fn integrity_for(bytes: &[u8]) -> String {
    let digest = Sha512::digest(bytes);
    format!("sha512-{}", BASE64_STANDARD.encode(digest))
}
