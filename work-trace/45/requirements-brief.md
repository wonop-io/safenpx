# Requirements Brief

## Acceptance Criteria

- Selected bin path is inside the verified extraction root.
- Selected bin digest is recorded before execution is considered.
- Generated shim behavior is either deterministic with digest evidence or
  refused with reason `shim_identity_mismatch` / `unsupported_closure`.
- Tests cover direct bin, package `bin` object, package `bin` string, generated
  shim candidate, and bin path escape.
- No test executes third-party package code.

## Verification

- `just test`

