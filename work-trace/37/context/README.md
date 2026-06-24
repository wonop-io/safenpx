# Issue 37 Context

## Source Inputs

- GitHub issue #37.
- `docs/milestones.md` M1 integrity verification deliverable.
- `ArtifactBytes` from issue #36.
- `ArtifactIdentity` and M1 reason vocabulary from issue #33.

## Assumptions

- M1 supports `sha512` npm integrity metadata first.
- Missing, malformed, unsupported, and mismatched integrity all fail closed
  with `M1Reason::IntegrityMismatch`.
- The stable artifact identity digest should use lowercase hexadecimal `sha512`
  of the exact downloaded bytes.

