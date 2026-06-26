# Issue 61 Progress

## 2026-06-26

- Moved issue #61 to `status:in-progress`.
- Created trace scaffold before implementation.
- Added a non-authoritative M3 inspect receipt shape with artifact identity,
  command identity, evidence summary, policy version, nullable timestamp, and
  redaction metadata.
- Wired `decision_receipt` into inspect JSON additively and kept
  execution-refusal JSON receipts null.
- Added tests for receipt shape, non-cache semantics, redaction metadata, and
  display/canonical identity separation.
- Documented that M3 receipts are not approvals, allow-list entries, or cache
  keys.
