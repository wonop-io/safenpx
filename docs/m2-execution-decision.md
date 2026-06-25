# M2 Execution Decision

M2 chooses `direct_extract` as the first alpha execution path.

The first runnable package class is intentionally small:

- exact-version npm package specs,
- a verified root artifact extracted by `safe-npx`,
- one deterministic selected bin inside the extracted root,
- selected bin bytes hashed before launch,
- forwarded arguments preserved as process argv,
- no lifecycle scripts,
- no dependency declarations unless a later milestone proves the full
  dependency closure,
- registry source agreement before execution preparation,
- no package-manager delegation and no raw `npx` fallback.

Everything outside that subset remains inspect-only or returns
`execution_refused`. If dependency closure cannot be proven, the required
reason is `unsupported_closure`.

## Compared Paths

### `direct_extract`

`direct_extract` runs from bytes that `safe-npx` downloaded, verified,
extracted, inspected, and selected. The #49 fixture prototype proves this can
work for a root-only subset without asking npm, `npx`, or another package
manager to reconstruct the command later.

This is not broad execution support. It is the smallest honest executable
surface: exact version, one selected root bin, no lifecycle scripts, no
dependencies, and fail-closed behavior for every unproven edge.

### `pinned_delegation`

`pinned_delegation` is rejected for M2. A pinned tarball does not prove the
bytes that a delegated package manager will execute. Delegation can reopen
metadata resolution, registry precedence, cache use, dependency install,
lifecycle execution, generated shims, package-manager configuration, ambient
environment, and command construction.

The #50 feasibility fixtures classify those authority gaps as
`execution_refused` with reason `unsupported_closure`.

### `inspect_only_alpha`

`inspect_only_alpha` remains the fallback for unsupported shapes and unproven
closures. It is not the global alpha choice because the direct-extract fixtures
prove a narrower executable subset without weakening the byte-identity
invariant.

## Required Refusals

`safe-npx` must refuse execution for:

- floating tags and ranges until tag-move races fail closed in the execution
  path,
- lifecycle scripts,
- dependency declarations outside a fully verified closure,
- ambiguous, missing, unsafe, or generated executable paths,
- registry source disagreement,
- metadata changes between inspection and execution preparation,
- cache identity mismatch,
- any package-manager handoff that can execute bytes not selected by
  `safe-npx`.

No selected path may delegate to raw `npx`.

## M5 Constraint

M5 execute-mode work is unblocked only for the `direct_extract` root-only
subset above. The M5 issue must keep broader execution behind explicit closure
proofs rather than expanding by fallback.
