# Inspect JSON Schema V0

## Premise

Inspect JSON is the agent and CI contract. It should carry enough structured
evidence for callers to stop, ask, deny, or retry without parsing human output.

## Top-Level Shape

M3 JSON v0 must expose these fields:

- `schema_version`
- `artifact`
- `command_intent`
- `source_context`
- `authority_context`
- `facts`
- `heuristics`
- `external_evidence`
- `attestations`
- `release_diff`
- `decision`
- `reasons`
- `required_next_action`
- `execution`
- `exit_code`

## V0 Rules

- `schema_version` is `0.1`.
- `execution` is null in inspect mode.
- `external_evidence`, `attestations`, and `release_diff` are present and null.
- Reserved fields do not implement hosted audits, attestations, or release
  diffs in M3.
- Enum spelling follows `docs/milestones.md`.

## Compatibility

- Additive fields are allowed in `0.x`.
- Enum additions require a schema bump.
- Enum semantic changes require a migration note.

## Fixture Expectations

Issue #5 should include minimum schema fixtures for:

- ask-style inspection
- unsupported input
- inspection/refusal failure

Issue #59 owns the broader golden fixture corpus and byte-stable output
coverage.
