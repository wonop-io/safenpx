# Deterministic Inspect JSON Goldens

## Scope

Extend the issue #5 JSON schema fixture infrastructure so it proves byte-stable
rendering for the broader M3 evidence surface.

## Fixture Scenarios

- Successful inspection with verified artifact identity.
- Unsupported input with no downloads.
- Integrity failure before extraction or execution.
- Lifecycle/dependency blockers surfaced as static evidence.
- Redacted authority context with token-like and host-path inputs.
- Missing optional registry metadata.

## Compatibility Rules

- Full rendered JSON must be compared against checked-in fixtures.
- Fixtures must use deterministic report builders, explicit paths, and no
  live network, current clock, locale, or temp-directory dependency.
- Enum spelling and semantic mappings must stay covered by tests.
- Schema compatibility documentation must include additive-field and enum-change
  rules used by the tests.

## Verification

- `cargo fmt --check`
- Focused inspect JSON schema tests.
- `just test`

