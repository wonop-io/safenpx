# M5 Requirements Brief

GitHub issue #7 is the parent/index issue for M5 execute mode alpha.

## Goal

Safely run the intended command only after policy allows the verified execution
closure. If the closure cannot be proven, the alpha must remain inspect-only or
return a stable refusal.

## Hard Constraints

- Execute mode stays inside the M2 `direct_extract` root-only subset.
- Exact-version package specs are the initial executable remote package shape.
- `safe-npx execute` is the only command path that may launch package code.
- Inspect and JSON inspect paths remain non-executing.
- No raw `npx`, `npm exec`, install hook, package-manager delegation, generated
  shim, dependency script, or lifecycle script may execute before proof.
- Approval cache behavior remains experimental or disabled until authority
  context golden tests pass.
- `latest` and moving specs must fail closed unless tag-race proof is complete.

## Required Outcomes

- Contracted execute CLI surface.
- Carried artifact identity from inspection through launch.
- Direct-extract root-only launch implementation.
- Interactive approval prompt with no-by-default behavior.
- JSON and agent behavior that remains inspect-first and explicit about execute.
- Experimental approval cache and authority-context cache tests.
- Moving-spec refusal behavior.
- Compatibility matrix, alpha distribution docs, help/reference coverage, and
  dogfood evidence.
- Closeout audit before #7 and the M5 milestone close.

## Verification

- Each child issue uses `just test` as its handoff verification.
- The closeout issue must also verify GitHub Actions on main before milestone
  closure.

