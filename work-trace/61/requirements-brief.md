# Requirements Brief

## Problem

M3 needs a stable inspect decision receipt shape so future local/shareable
records have somewhere to land, but it must not become an approval cache or imply
that an inspect receipt authorizes execution.

## Acceptance Mapping

- Receipt shape includes artifact digest, command intent, evidence summary,
  policy version, timestamp, and redaction metadata.
- Receipt shape does not implement approval-cache behavior in M3.
- Receipt identity fields avoid redacted display strings when canonical values
  are required.
- Receipt display fields do not expose tokens, home paths, or full environment
  details.
- Tests or fixtures cover receipt shape stability and redaction metadata.

## Constraints

- Use deterministic fixture timestamps in tests.
- Do not read or write an approval cache.
- Do not make receipts executable approvals.
- Preserve existing inspect JSON schema version unless the shape is additive.

