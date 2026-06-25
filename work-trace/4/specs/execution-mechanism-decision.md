# M2 Execution Mechanism Decision

## Decision

M2 chooses `direct_extract` as the first alpha execution path.

The first runnable package class is deliberately narrow:

- exact-version npm package specs only,
- a verified root artifact extracted by `safe-npx`,
- one deterministic selected bin inside the extracted root,
- selected bin bytes hashed before launch,
- forwarded arguments preserved as process argv without shell interpolation,
- no lifecycle scripts,
- no dependency declarations unless a later milestone proves the full
  dependency closure,
- registry source agreement between inspection and execution preparation,
- no package-manager delegation and no raw `npx` fallback.

Every other command shape remains inspect-only or returns `execution_refused`.
If dependency closure cannot be proven, the required decision is
`execution_refused` with reason `unsupported_closure`.

## Options Compared

### `direct_extract`

`direct_extract` executes from the artifact bytes that `safe-npx` downloaded,
verified, extracted, inspected, and selected. The #49 fixture prototype proves
the root-only subset can preserve the byte boundary for local fixtures: package
identity is exact, lifecycle and dependency declarations refuse, selected bin
identity is hashed before execution, and forwarded args are passed directly to
the child process.

This path is not yet a broad npm execution product. It is a proof-shaped alpha
path: exact versions, one root bin, fixture-backed execution semantics, and
fail-closed refusal for anything outside the verified closure.

### `pinned_delegation`

`pinned_delegation` is rejected for M2. Even when a root tarball is pinned,
delegating execution to npm or another package manager reopens authority that
`safe-npx` has not proven: metadata re-resolution, registry precedence,
mutable cache behavior, dependency install, lifecycle execution, generated
shims, package-manager config, and command reconstruction.

The #50 feasibility manifest maps every delegated authority gap to
`execution_refused` with reason `unsupported_closure`. The important result is
not merely that raw `npx` is unsafe; it is that any unproven package-manager
handoff can inspect one boundary and execute another.

### `inspect_only_alpha`

`inspect_only_alpha` remains the correct fallback for unsupported command
shapes, floating tags, unverified dependencies, lifecycle scripts, generated
shims, registry disagreement, and cache identity drift. It is rejected as the
global M2 choice because the #49 direct-extract prototype proves a smaller
runnable subset without weakening byte identity.

## Evidence Map

The M2 evidence tickets support this decision:

- #42 defines the closure and refusal vocabulary used by this decision.
- #9 proves inspection does not execute package-code canaries.
- #43 proves root artifact extraction for static inspection.
- #44 refuses lifecycle scripts and dependency declarations as closure blockers.
- #10 proves deterministic bin selection and forwarded-argument preservation.
- #45 treats generated shim identity as a closure boundary.
- #46 requires registry source agreement.
- #47 proves metadata, tag, tarball, cache, and delegation races fail closed.
- #48 seeds golden M2 outcomes across canary, lifecycle, dependency, registry,
  race, cache, shim, and closure fixtures.
- #49 proves the root-only direct-extract fixture execution subset.
- #50 rejects pinned delegation for M2.
- #51 wires `execution_refused` outputs for unproven closures.

Fixture evidence anchors:

- `m2-closure-fixture-manifest.txt` requires `unsupported_closure` for
  dependency closure gaps and refusal for lifecycle, registry, race, cache, and
  shim mismatches.
- `race-matrix-fixture-manifest.txt` keeps `exact_version_pinned` on the
  `direct_extract` path and refuses `pinned_delegation_unproven`.
- `bin-selection-fixture-manifest.txt` allows deterministic selected bins and
  refuses ambiguous, missing, or unsafe bin paths.
- `delegation-feasibility-manifest.txt` records the authority gaps that prevent
  package-manager delegation from being the first alpha path.

## Consequences

M5 may prototype execute mode only for the `direct_extract` root-only subset.
It must not add broad `latest` support, dependency installation, lifecycle
script execution, generated package-manager shims, or raw `npx` fallback before
those closures have their own proof.

When a user asks for an unsupported topology, `safe-npx` should still provide
inspection evidence where possible, then stop with the right refusal reason.
