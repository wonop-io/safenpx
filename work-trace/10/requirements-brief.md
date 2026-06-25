# Requirements Brief

## Acceptance Criteria

- Single-bin package selection is deterministic.
- Ambiguous bin returns `execution_refused` or `unsupported` with stable reason
  `ambiguous_bin`.
- Missing bin returns stable reason `missing_bin` without package execution.
- Scoped package bin selection is deterministic.
- Package-name/bin-name mismatch behavior is documented and covered by
  fixtures.
- Forwarded args are represented in command identity and JSON without
  normalization loss.

## Verification

- `just test`
- Bin-selection fixture tests.

