//! CLI-level integration coverage for the M4 decision contract.

use crate::{
    build_m2_execution_refusal_report, build_report_with_resolver, exit_code_for_report,
    render_m2_execution_refusal_report, render_report, Cli, ClosureCommandIdentity, M2Reason,
    NpmMetadataClient, RegistryHttpResponse, RegistryTransport, RegistryTransportError,
    RootArtifactResolver, TarballDownloader, TarballHttpResponse, TarballTransport,
    TarballTransportError, M4_ASK_REQUIRED_EXIT_CODE, M4_DENIED_EXIT_CODE,
    M4_EXECUTION_REFUSED_EXIT_CODE, M4_INSPECTION_ERROR_EXIT_CODE, M4_SUCCESS_EXIT_CODE,
    M4_UNSUPPORTED_EXIT_CODE,
};
use base64::prelude::{Engine as _, BASE64_STANDARD};
use flate2::{write::GzEncoder, Compression};
use serde_json::Value;
use sha2::{Digest, Sha512};
use std::cell::RefCell;
use tar::{Builder, Header};

#[derive(Debug)]
/// Stub registry transport with deterministic queued responses.
struct StubRegistryTransport {
    responses: RefCell<Vec<Result<RegistryHttpResponse, RegistryTransportError>>>,
}

impl StubRegistryTransport {
    /// Build a registry stub from responses consumed in order.
    fn new(responses: Vec<Result<RegistryHttpResponse, RegistryTransportError>>) -> Self {
        Self {
            responses: RefCell::new(responses),
        }
    }
}

impl RegistryTransport for StubRegistryTransport {
    /// Return the next queued registry response.
    fn get(&self, _url: &str) -> Result<RegistryHttpResponse, RegistryTransportError> {
        self.responses.borrow_mut().remove(0)
    }
}

#[derive(Debug)]
/// Stub tarball transport with deterministic queued responses.
struct StubTarballTransport {
    responses: RefCell<Vec<Result<TarballHttpResponse, TarballTransportError>>>,
}

impl StubTarballTransport {
    /// Build a tarball stub from responses consumed in order.
    fn new(responses: Vec<Result<TarballHttpResponse, TarballTransportError>>) -> Self {
        Self {
            responses: RefCell::new(responses),
        }
    }
}

impl TarballTransport for StubTarballTransport {
    /// Return the next queued tarball response.
    fn get_bytes(&self, _url: &str) -> Result<TarballHttpResponse, TarballTransportError> {
        self.responses.borrow_mut().remove(0)
    }
}

#[test]
/// Verifies inspect-mode decisions bind renderers, reasons, next actions, and exit codes.
fn decision_contract_covers_allow_ask_deny_unsupported_and_inspection_error() {
    let clean_package = package_tarball(
        r#"{
            "name":"create-example",
            "version":"1.2.3",
            "bin":{"create-example":"bin/create.js"}
        }"#,
    );
    assert_inspect_decision_case(InspectDecisionCase {
        argv: &[
            "safe-npx",
            "--decision",
            "allow",
            "inspect",
            "create-example@1.2.3",
        ],
        resolver_fixture: ResolverFixture::Verified(clean_package.clone()),
        expected_decision: "allow",
        expected_reasons: &["caller_requested_allow"],
        expected_next_action: "none",
        expected_exit_code: M4_SUCCESS_EXIT_CODE,
    });

    assert_inspect_decision_case(InspectDecisionCase {
        argv: &["safe-npx", "inspect", "create-example@1.2.3"],
        resolver_fixture: ResolverFixture::Verified(clean_package.clone()),
        expected_decision: "ask",
        expected_reasons: &["caller_requested_ask"],
        expected_next_action: "ask_user",
        expected_exit_code: M4_SUCCESS_EXIT_CODE,
    });

    let lifecycle_package = package_tarball(
        r#"{
            "name":"create-example",
            "version":"1.2.3",
            "scripts":{"postinstall":"node postinstall.js"}
        }"#,
    );
    assert_inspect_decision_case(InspectDecisionCase {
        argv: &[
            "safe-npx",
            "--decision",
            "allow",
            "--source-context",
            "ci",
            "inspect",
            "create-example@1.2.3",
        ],
        resolver_fixture: ResolverFixture::Verified(lifecycle_package),
        expected_decision: "ask",
        expected_reasons: &["caller_requested_allow", "lifecycle_script_present"],
        expected_next_action: "ask_user",
        expected_exit_code: M4_ASK_REQUIRED_EXIT_CODE,
    });

    assert_inspect_decision_case(InspectDecisionCase {
        argv: &[
            "safe-npx",
            "--decision",
            "deny",
            "inspect",
            "create-example@1.2.3",
        ],
        resolver_fixture: ResolverFixture::Verified(clean_package),
        expected_decision: "deny",
        expected_reasons: &["caller_requested_deny"],
        expected_next_action: "none",
        expected_exit_code: M4_DENIED_EXIT_CODE,
    });

    assert_inspect_decision_case(InspectDecisionCase {
        argv: &["safe-npx", "inspect", "create-example@latest"],
        resolver_fixture: ResolverFixture::Verified(b"unused".to_vec()),
        expected_decision: "unsupported",
        expected_reasons: &["unsupported_spec"],
        expected_next_action: "retry_narrower_command",
        expected_exit_code: M4_UNSUPPORTED_EXIT_CODE,
    });

    assert_inspect_decision_case(InspectDecisionCase {
        argv: &["safe-npx", "inspect", "create-example@1.2.3"],
        resolver_fixture: ResolverFixture::RegistryFailure,
        expected_decision: "inspection_error",
        expected_reasons: &["registry_error"],
        expected_next_action: "inspect_only",
        expected_exit_code: M4_INSPECTION_ERROR_EXIT_CODE,
    });
}

#[test]
/// Verifies execution-refused reports expose the same contract in human and JSON modes.
fn execution_refused_contract_reaches_human_json_and_exit_code() {
    let human_cli = Cli::parse_from([
        "safe-npx",
        "--m2-refusal",
        "unsupported-closure",
        "create-example@1.2.3",
    ]);
    let json_cli = Cli::parse_from([
        "safe-npx",
        "--json",
        "--m2-refusal",
        "unsupported-closure",
        "create-example@1.2.3",
    ]);
    let report =
        build_m2_execution_refusal_report(command_identity(), vec![M2Reason::UnsupportedClosure]);
    let human = render_m2_execution_refusal_report(&human_cli, &report).expect("render human");
    let json = render_m2_execution_refusal_report(&json_cli, &report).expect("render json");
    let value = serde_json::from_str::<Value>(&json).expect("valid json");

    assert_eq!(report.exit_code, M4_EXECUTION_REFUSED_EXIT_CODE);
    assert_eq!(value["decision"], "execution_refused");
    assert_eq!(value["required_next_action"], "inspect_only");
    assert_eq!(value["exit_code"], M4_EXECUTION_REFUSED_EXIT_CODE);
    assert!(value["reasons"]
        .as_array()
        .expect("reasons")
        .contains(&Value::String("unsupported_closure".to_string())));
    assert!(value["execution"].is_null());

    assert!(human.contains("Decision: execution_refused"));
    assert!(human.contains("Reasons: unsupported_closure"));
    assert!(human.contains("Required next action: inspect_only"));
    assert!(human.contains("Exit code: 14"));
    assert!(human.contains("No package code was executed."));
}

/// One CLI-shaped inspect decision scenario.
struct InspectDecisionCase {
    argv: &'static [&'static str],
    resolver_fixture: ResolverFixture,
    expected_decision: &'static str,
    expected_reasons: &'static [&'static str],
    expected_next_action: &'static str,
    expected_exit_code: i32,
}

/// Resolver fixture variants used to rebuild consumed stubs per renderer.
enum ResolverFixture {
    /// Verified package bytes served with matching integrity.
    Verified(Vec<u8>),
    /// Registry response that maps to inspection-error policy.
    RegistryFailure,
}

impl ResolverFixture {
    /// Build a fresh resolver for one renderer pass.
    fn build(&self) -> RootArtifactResolver<StubRegistryTransport, StubTarballTransport> {
        match self {
            Self::Verified(bytes) => verified_resolver(bytes),
            Self::RegistryFailure => registry_failure_resolver(),
        }
    }
}

/// Assert one inspect decision scenario through human and JSON renderers.
fn assert_inspect_decision_case(case: InspectDecisionCase) {
    let human_argv = case.argv.to_vec();
    let mut json_argv = case.argv.to_vec();
    json_argv.insert(1, "--json");

    let human_cli = Cli::parse_from(human_argv);
    let json_cli = Cli::parse_from(json_argv);
    let human_resolver = case.resolver_fixture.build();
    let json_resolver = case.resolver_fixture.build();
    let human_report = build_report_with_resolver(&human_cli, &human_resolver);
    let human = render_report(&human_cli, &human_report).expect("render human");

    let json_report = build_report_with_resolver(&json_cli, &json_resolver);
    let json = render_report(&json_cli, &json_report).expect("render json");
    let value = serde_json::from_str::<Value>(&json).expect("valid json");

    assert_eq!(exit_code_for_report(&human_report), case.expected_exit_code);
    assert_eq!(exit_code_for_report(&json_report), case.expected_exit_code);
    assert_eq!(value["decision"], case.expected_decision);
    assert_eq!(value["required_next_action"], case.expected_next_action);
    assert_eq!(value["exit_code"], case.expected_exit_code);
    assert_eq!(json_reasons(&value), case.expected_reasons);
    assert!(value["execution"].is_null());
    assert_eq!(
        human_report.inspect.execution_state.package_code_executed,
        false
    );
    assert_eq!(
        json_report.inspect.execution_state.package_code_executed,
        false
    );

    assert!(human.contains(&format!("Policy decision: {}", case.expected_decision)));
    assert!(human.contains(&format!(
        "Required next action: {}",
        case.expected_next_action
    )));
    assert!(human.contains(&format!("Exit code: {}", case.expected_exit_code)));
    for reason in case.expected_reasons {
        assert!(human.contains(reason));
    }
    assert!(human.contains("package code executed: false"));
}

/// Return top-level JSON policy reason strings.
fn json_reasons(value: &Value) -> Vec<&str> {
    value["reasons"]
        .as_array()
        .expect("json reasons")
        .iter()
        .filter_map(Value::as_str)
        .collect()
}

/// Return the deterministic command identity for execution-refused fixtures.
fn command_identity() -> ClosureCommandIdentity {
    ClosureCommandIdentity {
        requested: "create-example@1.2.3".to_string(),
        forwarded_args: Vec::new(),
    }
}

/// Build a resolver that serves verified package bytes.
fn verified_resolver(
    bytes: &[u8],
) -> RootArtifactResolver<StubRegistryTransport, StubTarballTransport> {
    resolver_with(metadata_body(&integrity_for(bytes)), bytes.to_vec())
}

/// Build a resolver whose registry request fails.
fn registry_failure_resolver() -> RootArtifactResolver<StubRegistryTransport, StubTarballTransport>
{
    RootArtifactResolver::new(
        NpmMetadataClient::public(StubRegistryTransport::new(vec![Ok(RegistryHttpResponse {
            status: 500,
            body: "{}".to_string(),
        })])),
        TarballDownloader::new(StubTarballTransport::new(Vec::new())),
    )
}

/// Build a resolver from explicit registry metadata and tarball bytes.
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

/// Return minimal npm metadata for the fixture package.
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

/// Return the npm sha512 integrity string for fixture bytes.
fn integrity_for(bytes: &[u8]) -> String {
    let digest = Sha512::digest(bytes);
    format!("sha512-{}", BASE64_STANDARD.encode(digest))
}

/// Build a gzipped npm package tarball containing the supplied package JSON.
fn package_tarball(package_json: &str) -> Vec<u8> {
    let mut gzip = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = Builder::new(&mut gzip);
        append_file(
            &mut builder,
            "package/package.json",
            package_json.as_bytes(),
        );
        append_file(
            &mut builder,
            "package/bin/create.js",
            b"#!/usr/bin/env node\n",
        );
        builder.finish().expect("tar builder should finish");
    }
    gzip.finish().expect("gzip encoder should finish")
}

/// Append a file to a fixture tarball.
fn append_file(builder: &mut Builder<&mut GzEncoder<Vec<u8>>>, path: &str, bytes: &[u8]) {
    let mut header = Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, path, bytes)
        .expect("fixture file should append");
}
