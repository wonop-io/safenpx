# Requirements Brief

## Source

- GitHub issue #71.
- M4 parent issue #6.
- #66 policy model, #67 thresholds, #68/#69 ask semantics, and #70 exit codes.

## Scope

- Define stable semantics for every `required_next_action` value.
- Map every canonical decision and major reason to a documented next action.
- Keep `explicit_override` semantic-only in M4.
- Ensure JSON and human output agree.

## Acceptance Criteria

- Every decision enum value maps to a documented default next action.
- Reason-specific mappings are tested for unsupported specs, lifecycle scripts,
  unsupported closure, integrity mismatch, and inspection errors.
- JSON fixtures include representative next-action cases.
- Human output explains the next safe action without implying the package is
  safe.
- Reserved evidence fields remain null unless implemented by their own
  milestone.

