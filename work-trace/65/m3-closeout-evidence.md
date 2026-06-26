# M3 Closeout Evidence

Date: 2026-06-26

GitHub issue: https://github.com/wonop-io/safenpx/issues/65

Milestone: M3: Inspect Evidence And JSON V0

## Issue State

Open M3 issues before this closeout commit:

- #65 M3: Audit and close inspect evidence milestone

Closed M3 implementation and documentation issues:

- #5 M3: Define inspect JSON schema v0 for agents and CI
- #11 M3: Extract root package evidence without execution
- #12 M3: Define authority-context redaction rules
- #53 M3: Plan inspect evidence ticket map
- #54 M3: Wire inspect mode pipeline without execution
- #55 M3: Carry no-package-code canaries into inspect mode
- #56 M3: Extract registry metadata evidence for inspect reports
- #57 M3: Model inspect facts heuristics and decisions separately
- #58 M3: Render human inspect report from shared evidence
- #59 M3: Add deterministic inspect JSON golden fixtures
- #60 M3: Model caller-declared source context
- #61 M3: Define inspect decision receipt shape
- #62 M3: Reserve external evidence attestation and release-diff fields
- #63 M3: Measure provisional inspect latency budgets
- #64 M3: Document inspect-first POC workflow and limits

## Deliverable Audit

| M3 deliverable | Evidence |
| --- | --- |
| Inspect mode resolves, downloads, verifies, extracts evidence, and stops | `crates/safe-npx/src/m3_inspect.rs`, `crates/safe-npx/src/report.rs`, `crates/safe-npx/src/m3_inspect_tests.rs` |
| Static extraction from registry metadata, tarball metadata, and `package.json` | `crates/safe-npx/src/registry_evidence.rs`, `crates/safe-npx/src/extraction.rs`, `crates/safe-npx/src/package_evidence.rs` |
| Human report separates facts, heuristics, decision, and authority context | `crates/safe-npx/src/report_inspect.rs`, `crates/safe-npx/src/report_optional_evidence.rs`, `crates/safe-npx/src/inspect_human_golden_tests.rs` |
| Versioned JSON schema with deterministic golden fixtures | `crates/safe-npx/src/inspect_json_schema.rs`, `crates/safe-npx/src/inspect_json_schema_tests.rs`, `crates/safe-npx/src/inspect_json_golden_tests.rs` |
| JSON compatibility rule documented | `docs/inspect-json-schema-v0.md`, `crates/safe-npx/src/inspect_json_golden_tests.rs` |
| V0 JSON fields present | `crates/safe-npx/src/inspect_json_schema.rs`, `crates/safe-npx/fixtures/inspect-json-schema-v0-*.json` |
| Reserved fields remain null | `crates/safe-npx/src/inspect_json_schema_tests.rs`, `crates/safe-npx/src/inspect_json_golden_tests.rs`, `docs/inspect-json-schema-v0.md` |
| Decision receipt shape defined without approval-cache semantics | `crates/safe-npx/src/inspect_receipt.rs`, `crates/safe-npx/src/inspect_receipt_tests.rs`, `docs/inspect-json-schema-v0.md` |
| Privacy and redaction rules for reports and JSON | `crates/safe-npx/src/redaction.rs`, `crates/safe-npx/src/authority_context.rs`, `crates/safe-npx/src/report_redaction_tests.rs` |

## Acceptance Audit

| M3 acceptance criterion | Evidence |
| --- | --- |
| Inspect mode never runs package binaries, lifecycle scripts, or dependency scripts | `crates/safe-npx/src/m3_inspect_canary_tests.rs`, `crates/safe-npx/src/canary.rs`, `crates/safe-npx/fixtures/canary-fixture-manifest.txt` |
| JSON output is deterministic for fixture packages and has compatibility tests | `crates/safe-npx/src/inspect_json_schema_tests.rs`, `crates/safe-npx/src/inspect_json_golden_tests.rs`, `crates/safe-npx/fixtures/inspect-json-schema-v0-*.json` |
| Reports never print secret values, private registry tokens, or full sensitive environment details | `crates/safe-npx/src/report_redaction_tests.rs`, `crates/safe-npx/fixtures/authority-redaction-fixture-manifest.txt` |
| Authority context includes registry source, package scope, command intent, cwd trust class, and source context categories | `crates/safe-npx/src/authority_context.rs`, `crates/safe-npx/src/inspect_model_tests.rs` |
| Authority context examples cover terminal/CI/agent, trusted/temp cwd, and public/scoped registry categories | `crates/safe-npx/src/inspect_model_tests.rs`, `docs/inspect-first-poc.md` |
| Redacted display output stays separate from canonical/hashed identity fields | `crates/safe-npx/src/authority_context.rs`, `crates/safe-npx/src/inspect_receipt.rs`, `crates/safe-npx/src/inspect_json_golden_tests.rs` |
| Reports plainly state what `safe-npx` catches and does not catch | `docs/inspect-first-poc.md`, `README.md`, `crates/safe-npx/src/report.rs` |
| Provisional latency budgets are measured: cold public-package inspect under five seconds and warm inspect under one second | Cold live `just latency-live` on `is-number@7.0.0`: `real 1.67`; warm fixture `just latency-fixture`: `total_ms: 1`; `crates/safe-npx/src/inspect_latency.rs`, `crates/safe-npx/src/inspect_latency_tests.rs`, `docs/inspect-latency-budgets.md` |

## Explicit Deferrals

- Reserved `external_evidence`, `attestations`, and `release_diff` fields stay
  null in V0. Hosted audits, third-party attestations, and release diffs require
  later schema and provenance work.
- Decision receipts are non-authoritative in M3. Approval-cache semantics,
  validation, replay, expiry, and execution effects are later milestone work.
- Live public npm latency is documented as an optional manual end-to-end
  measurement, not a CI gate. Cold live measurement for this closeout was
  `is-number@7.0.0` at `real 1.67`, under the five-second budget. CI validates
  budget constants, evidence shape, and deterministic phase accounting.
- Live public npm phase breakdown is deferred. M3 records phase breakdown for
  fixture-backed inspect only; live phase instrumentation can be added later if
  public npm regressions become hard to diagnose.
- General package execution remains outside M3. Execution support remains
  constrained by M2 and later M5 work.

## Verification

Local verification before closeout:

- `just latency-fixture` passed and printed warm fixture phase evidence.
- `just latency-live` passed for `is-number@7.0.0` with `real 1.67`, under the
  five-second cold public-package budget.
- `just test` passed with 203 Rust tests, 1 ignored manual latency test, Rust
  documentation coverage 82%, line coverage 93.46%, and Bazel tests passing.

Latest GitHub Actions evidence for closeout:

- `docs(65): record m3 closeout evidence`
- Run ID: `28235127946`
- Status: success
- Jobs: Policy Preflight, Rust And Bazel

Final closeout state:

- GitHub issue #65 is closed.
- GitHub milestone #4, `M3: Inspect Evidence And JSON V0`, is closed.
- M3 has zero open issues and sixteen closed issues.
