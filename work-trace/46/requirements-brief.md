# Requirements Brief

## Acceptance Criteria

- Public npm default registry is recorded deterministically.
- Scoped package registry selection is tested with local config fixtures.
- Conflicting registry sources fail closed with reason
  `registry_precedence_mismatch`.
- M2 does not claim broad private registry support beyond inspect-only smoke
  coverage.
- Tests do not contact private registries or require user machine npm config.

## Verification

- `just test`
