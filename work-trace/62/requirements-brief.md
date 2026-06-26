# Requirements Brief

## Problem

M3 needs stable JSON slots for later hosted audit, attestation, and release-diff
work, while being brutally clear that those systems do not exist in V0 and are
not queried by inspect mode.

## Acceptance Mapping

- JSON includes `external_evidence`, `attestations`, and `release_diff` as null
  in V0.
- Tests prove the reserved fields remain null for every M3 fixture output.
- No hosted audit, attestation, or release-diff lookup code is introduced.
- Documentation states the fields are reserved for later milestones.
- Human reports do not imply hosted audits, attestations, or release diffs were
  checked.

## Constraints

- Keep all evidence deterministic and local.
- Preserve schema version `0.1`.
- Do not add network clients, registry calls, or hosted audit placeholders that
  can be mistaken for implemented behavior.
- Keep Cargo and Bazel wiring synchronized.

