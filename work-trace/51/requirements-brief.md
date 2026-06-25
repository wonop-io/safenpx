# Requirements Brief

## Acceptance Criteria

- Human output clearly says execution was refused and why.
- JSON output has stable machine-readable decision and reason fields for M2
  refusal.
- `unsupported_closure`, `ambiguous_bin`, `missing_bin`,
  `lifecycle_script_present`, `registry_precedence_mismatch`,
  `cache_identity_mismatch`, and `shim_identity_mismatch` are covered by tests.
- Non-interactive mode stops with reason `non_interactive_stop` where user
  input would be required.
- Refusal output never suggests falling back to raw `npx`.
- Output reason semantics match the completed proof tickets that introduced
  each reason.

## Verification

- `just test`

## Dependencies

- #42: execution closure contracts and reason vocabulary.
- #44: lifecycle and dependency blocker classification.
- #10: deterministic bin selection and forwarded args.
- #45: selected-bin and shim byte identity.
- #46: registry precedence evidence.
- #47: resolution-to-execution race matrix.

