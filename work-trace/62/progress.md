# Issue 62 Progress

## 2026-06-26

- Moved issue #62 to `status:in-progress`.
- Created trace scaffold before implementation.
- Confirmed the schema already serialized `external_evidence`, `attestations`,
  and `release_diff` as null in V0.
- Added fixture-level assertions that base and expanded M3 JSON outputs keep
  all reserved evidence fields null.
- Added human-output assertions that M3 reports do not claim hosted audits,
  attestations, or release-diff checks.
- Updated schema documentation to state no hosted audit registry,
  attestation source, or release-diff evidence is queried in M3.
- Ran prior-commit red-team/blue-team review. Round 1 found that JSON null
  assertions treated missing keys as null.
- Hardened reserved-field assertions to require keys to be present and null.
