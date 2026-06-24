# Issue 37 Requirements Brief

## Facts

- Issue #35 extracts `ResolvedPackage.integrity` from npm metadata.
- Issue #36 returns unmodified tarball bytes as `ArtifactBytes`.
- Existing contracts include `ArtifactIdentity` and
  `M1Reason::IntegrityMismatch`.
- npm integrity metadata commonly uses Subresource Integrity syntax such as
  `sha512-<base64 digest>`.

## Requirements

- Parse supported integrity metadata and verify it against downloaded bytes.
- Compute a deterministic digest identity for the verified bytes.
- Fail closed on mismatched bytes, missing integrity, malformed integrity, and
  unsupported integrity algorithms.
- Preserve the expected `deny` / `integrity_mismatch` outcome for mismatches.

## Non-Goals

- Tar archive extraction.
- Dependency integrity verification.
- Multiple integrity entries or algorithm preference negotiation beyond the
  first M1 supported algorithm.
- Human/JSON report integration; issue #39 owns report wiring.

