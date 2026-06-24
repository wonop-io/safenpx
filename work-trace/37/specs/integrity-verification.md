# Integrity Verification

## Behavior

- Accept `ArtifactBytes` and the expected npm integrity string.
- Support `sha512-<base64 digest>` integrity metadata.
- Decode the expected digest from base64.
- Compute SHA-512 over the exact downloaded bytes.
- Compare expected and actual digest bytes.
- Return `ArtifactIdentity` only when verification succeeds.
- Use lowercase hexadecimal SHA-512 as the stable artifact identity digest.

## Error Mapping

- Mismatched bytes return `Decision::Deny` and
  `M1Reason::IntegrityMismatch`.
- Missing integrity returns `Decision::Deny` and
  `M1Reason::IntegrityMismatch`.
- Malformed integrity returns `Decision::Deny` and
  `M1Reason::IntegrityMismatch`.
- Unsupported integrity algorithms return `Decision::Deny` and
  `M1Reason::IntegrityMismatch`.

## Test Requirements

- Tests cover valid integrity.
- Tests cover integrity mismatch.
- Tests cover missing integrity.
- Tests cover malformed integrity.
- Tests cover unsupported integrity algorithm.
- Tests prove repeated verification of the same bytes produces the same digest
  identity.

