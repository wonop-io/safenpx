# Requirements Brief

## Acceptance Criteria

- Design note explicitly compares `direct_extract`, `pinned_delegation`, and
  `inspect_only_alpha`.
- Decision record states one of `direct_extract`, `pinned_delegation`, or
  `inspect_only_alpha`.
- Decision is backed by fixture evidence from canary, race, bin, registry
  precedence, cache, shim, lifecycle, and dependency-closure tickets.
- Chosen path cannot inspect one package version and execute another.
- If dependency closure cannot be proven, the decision requires
  `execution_refused` with reason `unsupported_closure`.
- No selected path delegates to raw `npx` as a fallback.
- M5 issue #7 is updated with the chosen M2 outcome.

## Verification

- `just test`
- Decision record review against completed M2 evidence tickets.

