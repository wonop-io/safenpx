# Reserved Evidence Fields

## Current V0 Contract

The M3 JSON envelope exposes three future-facing fields:

- `external_evidence`
- `attestations`
- `release_diff`

In V0, each field must serialize as `null`. A non-null value would imply that
safe-npx contacted or consumed a hosted audit registry, attestation source, or
release-diff service.

## Human Output

Human inspect output should focus on evidence that was actually collected. It
may mention the reserved fields only as unavailable or reserved; it must not
claim that external audits, attestations, or release diffs were checked.

## Test Shape

- Assert all schema and expanded JSON golden fixtures have the three fields.
- Assert all three fields are null in every M3 fixture.
- Keep tests local and deterministic.

