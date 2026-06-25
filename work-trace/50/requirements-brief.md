# Requirements Brief

## Acceptance Criteria

- Analysis lists every delegation step that could change executed bytes.
- Any unresolved identity gap maps to `inspect_only_alpha` or refusal, not
  fallback.
- Local probes are deterministic and do not contact live npm unless explicitly
  stubbed or mocked.
- If pinned delegation is rejected, the rejection is documented with concrete
  proof gaps.
- If pinned delegation remains viable, required proof obligations become
  follow-up issues before M5.

## Verification

- `just test`
- Design-note review in #4.

## Dependencies

- #42: M2 closure contracts and reason vocabulary.
- #46: registry and `.npmrc` precedence agreement.
- #47: resolution-to-execution race matrix fixtures.

