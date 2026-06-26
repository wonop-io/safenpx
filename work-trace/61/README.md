# Issue 61 Work Trace

GitHub issue: https://github.com/wonop-io/safenpx/issues/61

This trace covers the M3 inspect decision receipt shape. The receipt is a local
or shareable record shape only; M3 must not implement approval-cache semantics.

## Scope

- Define fields for artifact digest, command intent, evidence summary, policy
  version, timestamp, and redaction metadata.
- Keep canonical identity values separate from redacted display values.
- Make the receipt inspect-oriented and non-authoritative until later policy
  milestones define semantics.

