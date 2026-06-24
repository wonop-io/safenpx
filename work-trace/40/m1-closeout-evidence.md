# M1 Closeout Evidence

Date: 2026-06-24

## Ticket State

| Issue | Status | Evidence |
| --- | --- | --- |
| #33 Define resolver data contracts and reason vocabulary | Closed | `contracts.rs` defines command intent, parser states, resolved package, artifact identity, and M1 reasons. |
| #2 Parse supported exact-version package specs | Closed | `parser.rs` supports exact unscoped and scoped specs and rejects unsupported shapes. |
| #34 Implement unsupported-spec refusal behavior | Closed | `report.rs` and parser contracts include refusal reasons and `downloaded=false`. |
| #38 Add malformed-spec no-network test harness | Closed | `inspect.rs` no-network harness covers malformed and unsupported specs. |
| #8 Seed resolver and artifact fixture manifest | Closed | `fixtures.rs` consumes parser, registry, artifact, malformed, and forwarded-arg fixture rows. |
| #35 Implement npm metadata client and error mapping | Closed | `registry.rs` resolves public npm metadata and maps missing package/version/registry errors. |
| #36 Download root tarball without execution | Closed | `download.rs` downloads raw bytes only and has side-effect counter tests. |
| #37 Verify npm integrity and compute artifact identity | Closed | `integrity.rs` verifies SHA-512 SRI and returns stable digest identity. |
| #3 Resolve and verify root npm artifact identity | Closed | `resolver.rs` composes metadata, tarball download, and integrity verification. |
| #39 Wire M1 evidence into human and JSON reports | Closed | `report.rs` emits no-download, verified, and failed M1 evidence in human and JSON output. |
| #40 Audit and close resolver milestone acceptance | Open during audit | This document records the closeout evidence. |

Open M1 search before closing #40:

```text
#40 M1: Audit and close resolver milestone acceptance
```

The repository does not currently have a formal GitHub milestone object named
`M1`; M1 closure is tracked through the documented milestone and issue title
prefixes.

## Deliverables

| M1 deliverable | Evidence |
| --- | --- |
| Package-spec parser with supported/unsupported matrix | `parser.rs` classifies exact `name@version` and `@scope/name@version`; rejects unversioned names, ranges, git URLs, local paths, tarball URLs, aliases, multiple specs, and npm exec variants. |
| npm registry client for public package metadata | `registry.rs` has `NpmMetadataClient`, public registry URL handling, metadata URL encoding, exact-version extraction, and stable error mapping. |
| Exact-version resolution | `registry.rs` resolves exact version metadata; `resolver.rs` accepts a supported `PackageSpec` and returns verified root artifact evidence. |
| Tarball download without lifecycle execution | `download.rs` fetches tarball bytes through `TarballTransport` and never extracts or invokes package managers; tests track zero package-manager, extraction, binary, lifecycle, and dependency-script calls. |
| Integrity verification and digest identity | `integrity.rs` verifies `sha512-...` SRI, returns `ArtifactIdentity`, and denies malformed/missing/unsupported/mismatched integrity. |
| Fixture manifest seed | `crates/safe-npx/fixtures/m1-fixture-manifest.txt` and `fixtures.rs` cover parser, registry errors, integrity mismatch, malformed specs, and forwarded args. |
| Unsupported-spec refusal messages | `report.rs` renders rejected input, reason, category, and `downloaded=false` for unsupported/malformed specs. |

## Acceptance Criteria

| Criterion | Evidence |
| --- | --- |
| Supported specs resolve to exact package name, version, registry, tarball URL, and digest. | `resolver::tests::resolves_verified_root_artifact` and `report_tests::renders_json_for_agents` assert resolved metadata and digest output. |
| Integrity mismatch returns `deny` and exits without execution. | `integrity::tests::denies_integrity_mismatch`, `resolver::tests::maps_integrity_mismatch_to_deny`, and `report_tests::renders_json_for_integrity_mismatch`. |
| Unsupported specs fail closed and never silently call raw `npx`. | `parser::tests::rejects_unsupported_specs_without_downloads`, `inspect::tests::unsupported_specs_do_not_reach_network_hooks`, and report refusal tests. |
| Malformed specs cause no network calls. | `inspect::tests::malformed_specs_do_not_reach_network_hooks` and no-network fixtures. |
| Resolver fixtures cover scoped packages, missing packages, missing versions, registry errors, malformed specs, integrity mismatch, and forwarded args. | Parser, registry, artifact, and no-network fixture tests in `fixtures.rs`; parser tests cover scoped exact specs and forwarded args are covered by report tests. |

## Verification

Local verification before this evidence commit:

```text
just test
57 tests passed
line coverage: 92.93%
bazel test //... passed
```

Latest main CI before this evidence commit:

```text
https://github.com/wonop-io/safenpx/actions/runs/28109670668
status: success
head: f74f2a782e4744372bd7aa07d6f7e5a9bb97499a
```

## Follow-Up Boundaries

No M1 deliverable needs a later follow-up to be considered complete. The
following are intentionally outside M1 and are already represented by later
milestone issues:

- M2 execution closure and byte-identity proof.
- M2 no-package-code-ran canary harness.
- M3 full inspect JSON schema and root package evidence extraction.
- M4 policy and exit semantics.
- M5 alpha execution or inspect-only decision.
