# M2 Closeout Audit

## Result

M2 is ready to close after #52 completes.

All M2 implementation issues are closed. The only open issue in the GitHub M2
milestone is this closeout issue. M5 remains open as future execute-mode alpha
work and is explicitly constrained by the #4 decision record.

## Execution Decision

M2 chooses `direct_extract` for the first alpha execution path.

The first runnable class is exact-version package specs, a verified root
artifact extracted by `safe-npx`, one deterministic selected root bin,
selected-bin bytes hashed before launch, forwarded argv preserved, no lifecycle
scripts, no dependency declarations unless the full dependency closure is
proven, registry source agreement, and no package-manager delegation.

Unsupported topologies remain inspect-only or return `execution_refused`.
Unproven dependency closure must return reason `unsupported_closure`.

Evidence:

- `docs/m2-execution-decision.md`
- `work-trace/4/specs/execution-mechanism-decision.md`
- GitHub issue #7 update:
  <https://github.com/wonop-io/safenpx/issues/7#issuecomment-4797916806>

## Deliverable Matrix

| M2 deliverable | Status | Evidence |
| --- | --- | --- |
| Execution-closure design note comparing direct execution, pinned local tarball execution, and package-manager delegation. | Complete | #4, `docs/m2-execution-decision.md`, `work-trace/4/specs/execution-mechanism-decision.md` |
| Decision record choosing `direct_extract`, `pinned_delegation`, or `inspect_only_alpha`. | Complete | #4 chooses `direct_extract`; M5 #7 updated with the outcome. |
| No-package-code-ran canary harness using sentinel files, environment markers, and blocked network attempts. | Complete | #9, `crates/safe-npx/src/canary.rs`, canary fixture manifest tests. |
| Race matrix for resolution time versus execution time. | Complete | #47, `crates/safe-npx/fixtures/race-matrix-fixture-manifest.txt`, race matrix tests. |
| Bin selection rules and refusal behavior for ambiguous or missing bins. | Complete | #10, `crates/safe-npx/src/bin_selection.rs`, bin-selection fixture manifest. |
| Bin fixtures for single bin, multiple bins without explicit selection, missing bin, scoped package bin, and forwarded args preserved exactly. | Complete | #10, `crates/safe-npx/fixtures/bin-selection-fixture-manifest.txt`. |
| Registry and `.npmrc` precedence tests for public npm and scoped inspect-only smoke cases. | Complete | #46, `crates/safe-npx/src/registry_precedence.rs`. |
| Closure fixtures for tag moves, cache poisoning, dependency lifecycle escape, generated shims, and selected binaries. | Complete | #48 plus #45 and #47 fixture manifests. |
| Direct execution proof for a first alpha subset. | Complete | #49, `work-trace/49/specs/direct-extract-fixture-execution.md`, direct execution tests. |
| Pinned delegation feasibility analysis. | Complete | #50, `crates/safe-npx/fixtures/delegation-feasibility-manifest.txt`. |
| Structured refusal output for unproven closures. | Complete | #51, report tests for M2 refusal output. |

## Acceptance Criteria Matrix

| M2 acceptance criterion | Status | Direct evidence |
| --- | --- | --- |
| Inspection cannot run package binaries, lifecycle scripts, or dependency scripts. | Complete | #9 canary harness; #43 static extraction; #44 lifecycle/dependency blockers. |
| The selected execution mechanism cannot inspect one version and execute another. | Complete | #4 decision restricts execution to direct extracted bytes; #47 race matrix; #49 direct-extract fixture execution; #50 rejects delegation. |
| Registry metadata changes between inspection and execution fail closed. | Complete | #47 `metadata_changed` and `tag_moved_latest`; #51 refusal output. |
| Cache entries, generated shims, selected bins, dependencies, and lifecycle scripts cannot escape the verified closure. | Complete | #45 executable identity and shim refusal; #44 blockers; #47 cache races; #49 direct execution refusals. |
| If full dependency closure cannot be proven, execution returns `execution_refused` with reason `unsupported_closure`. | Complete | #42 closure contracts; #48 `unsupported_dependency_closure`; #50 delegation gaps; #51 JSON/human refusal output. |
| If the executable subset is too narrow for useful alpha execution, the alpha ships as inspect-only rather than weakening the invariant. | Complete | #4 chooses narrow `direct_extract`, keeps inspect-only/refusal fallback, and constrains M5 #7. |

## Issue State

Closed M2 issues:

- #41 planned the M2 ticket map.
- #42 defined closure contracts and vocabulary.
- #9 built the no-package-code-ran canary harness.
- #43 extracted root artifacts safely for static inspection.
- #48 seeded M2 closure fixture manifests and golden outcomes.
- #44 detected lifecycle scripts and dependency declarations as blockers.
- #10 defined bin selection and forwarded-argument fixtures.
- #45 modeled selected bin and generated shim byte identity.
- #46 proved registry and `.npmrc` precedence agreement.
- #47 built resolution-to-execution race fixtures.
- #51 wired `execution_refused` outputs for unproven closures.
- #49 prototyped direct-extract root-only execution with local fixtures.
- #50 evaluated and rejected pinned delegation for M2.
- #4 chose the execution mechanism and updated M5.

Open M2 issues before this closeout:

- #52 only.

Open future execution work:

- #7 remains open under M5 and is not an M2 implementation issue.

## Deferred Follow-Ups

The following are intentionally deferred beyond M2:

- broad execute-mode productization,
- full dependency closure execution,
- lifecycle-script execution,
- generated package-manager shim execution,
- broad `latest` execution support,
- package-manager delegation,
- hosted audit records,
- broad private registry support.

Each deferred item is covered by later milestone scope, primarily M5 and beyond,
and must remain fail-closed until it has fixture-backed proof.

## Verification

Local verification for the final M2 decision commit:

- `just test` passed.
- Policy preflight passed.
- `cargo fmt --all -- --check` passed.
- `cargo test` passed with 143 tests.
- `cargo llvm-cov --workspace --all-targets --fail-under-lines 80` passed at
  91.75% line coverage.
- `bazel test //...` passed.

CI verification:

- GitHub Actions run
  <https://github.com/wonop-io/safenpx/actions/runs/28161899701> passed on
  `main`.

Review verification:

- #4 prior-commit red-team review: no attacks.
- #4 blue-team review: no conceded attacks.
- #4 judge review: no findings.
- Re-review count: 0.
