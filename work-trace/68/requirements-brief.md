# Requirements Brief

## Source

- GitHub issue #68.
- M4 threshold and policy model from issues #66 and #67.

## Scope

- Define interactive ask semantics for policy-required cases.
- Return `ask` when policy requires human confirmation.
- Preserve inspect-only behavior: no package code executes after an ask.
- Make human output state why the question is required.
- Keep approval caching out of scope until M5.

## Acceptance Criteria

- Interactive policy-required cases return decision `ask`.
- Human output states why the question is required.
- JSON output carries `decision: ask` and `required_next_action: ask_user`.
- Inspect mode does not execute package code after an ask decision.
- Tests cover lifecycle script, recent publish, and large package ask cases.

