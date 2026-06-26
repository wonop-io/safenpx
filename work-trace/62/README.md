# Issue 62 Work Trace

GitHub issue: https://github.com/wonop-io/safenpx/issues/62

This trace covers M3 reserved JSON fields for future hosted external evidence,
attestations, and release-diff evidence.

## Scope

- Keep `external_evidence`, `attestations`, and `release_diff` present and null
  in the V0 JSON envelope.
- Prove fixture outputs do not populate those fields in M3.
- Document that the fields are reserved placeholders, not live hosted lookups.

