//! Optional local fixture latency measurement for M3 inspect mode.

use crate::{
    build_report_with_resolver, extract_for_inspect, inspect_latency_evidence, render_report, Cli,
    InspectLatencyPhases, InspectLatencyProfile, NpmMetadataClient, RegistryHttpResponse,
    RegistryTransport, RegistryTransportError, RootArtifactResolver, TarballDownloader,
    TarballHttpResponse, TarballTransport, TarballTransportError,
};
use base64::prelude::{Engine as _, BASE64_STANDARD};
use flate2::{write::GzEncoder, Compression};
use sha2::{Digest, Sha512};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use tar::{Builder, Header};

#[test]
#[ignore = "manual fixture timing: cargo test -p safe-npx measure_fixture_inspect_latency -- --ignored --nocapture"]
/// Prints fixture-backed inspect latency evidence without live network access.
fn measure_fixture_inspect_latency() {
    let tarball = package_tarball(
        r#"{
            "name":"create-example",
            "version":"1.2.3",
            "bin":{"create-example":"bin/create.js"},
            "scripts":{"postinstall":"node postinstall.js"},
            "dependencies":{"left-pad":"1.3.0"}
        }"#,
    );
    let download_ms = Rc::new(RefCell::new(0_u128));
    let resolver = verified_resolver(&tarball, Rc::clone(&download_ms));
    let crate::PackageSpecParse::Supported(package_spec) =
        crate::classify_package_spec("create-example@1.2.3")
    else {
        panic!("fixture spec should be supported");
    };

    let resolve_start = Instant::now();
    let verified = resolver
        .resolve(&package_spec)
        .expect("fixture artifact should resolve");
    let resolve_total_ms = resolve_start.elapsed().as_millis();
    let download_phase_ms = *download_ms.borrow();

    let extract_start = Instant::now();
    extract_for_inspect(&verified.artifact_bytes, &verified.artifact_identity)
        .expect("fixture artifact should extract");
    let extract_ms = extract_start.elapsed().as_millis();

    let render_resolver = verified_resolver(&tarball, Rc::new(RefCell::new(0)));
    let cli = Cli::parse_from(["safe-npx", "inspect", "create-example@1.2.3"]);
    let report = build_report_with_resolver(&cli, &render_resolver);
    let render_start = Instant::now();
    let output = render_report(&cli, &report).expect("fixture report should render");
    let render_ms = render_start.elapsed().as_millis();
    assert!(output.contains("m3_inspect"));

    let evidence = inspect_latency_evidence(
        InspectLatencyProfile::WarmFixture,
        InspectLatencyPhases {
            resolve_ms: resolve_total_ms.saturating_sub(download_phase_ms),
            download_ms: download_phase_ms,
            extract_ms,
            render_ms,
        },
    );

    println!(
        "{}",
        serde_json::to_string_pretty(&evidence).expect("latency evidence should serialize")
    );
}

#[derive(Debug)]
/// Registry transport fixture that returns scripted metadata responses.
struct StubRegistryTransport {
    responses: RefCell<Vec<Result<RegistryHttpResponse, RegistryTransportError>>>,
}

impl StubRegistryTransport {
    /// Build a registry fixture from ordered responses.
    fn new(responses: Vec<Result<RegistryHttpResponse, RegistryTransportError>>) -> Self {
        Self {
            responses: RefCell::new(responses),
        }
    }
}

impl RegistryTransport for StubRegistryTransport {
    /// Return the next scripted metadata response.
    fn get(&self, _url: &str) -> Result<RegistryHttpResponse, RegistryTransportError> {
        self.responses.borrow_mut().remove(0)
    }
}

#[derive(Debug)]
/// Tarball transport fixture that times scripted artifact responses.
struct TimedTarballTransport {
    responses: RefCell<Vec<Result<TarballHttpResponse, TarballTransportError>>>,
    download_ms: Rc<RefCell<u128>>,
}

impl TimedTarballTransport {
    /// Build a timed tarball fixture from ordered responses.
    fn new(
        responses: Vec<Result<TarballHttpResponse, TarballTransportError>>,
        download_ms: Rc<RefCell<u128>>,
    ) -> Self {
        Self {
            responses: RefCell::new(responses),
            download_ms,
        }
    }
}

impl TarballTransport for TimedTarballTransport {
    /// Return the next scripted tarball response and record transport latency.
    fn get_bytes(&self, _url: &str) -> Result<TarballHttpResponse, TarballTransportError> {
        let started = Instant::now();
        let response = self.responses.borrow_mut().remove(0);
        *self.download_ms.borrow_mut() += started.elapsed().as_millis();
        response
    }
}

/// Build a resolver that serves verified bytes for the fixture package.
fn verified_resolver(
    bytes: &[u8],
    download_ms: Rc<RefCell<u128>>,
) -> RootArtifactResolver<StubRegistryTransport, TimedTarballTransport> {
    RootArtifactResolver::new(
        NpmMetadataClient::public(StubRegistryTransport::new(vec![Ok(RegistryHttpResponse {
            status: 200,
            body: metadata_body(&integrity_for(bytes)),
        })])),
        TarballDownloader::new(TimedTarballTransport::new(
            vec![Ok(TarballHttpResponse {
                status: 200,
                bytes: bytes.to_vec(),
            })],
            download_ms,
        )),
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
