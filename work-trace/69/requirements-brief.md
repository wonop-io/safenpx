# Requirements Brief

## Source

- GitHub issue #69.
- #67 policy thresholds.
- #68 interactive ask semantics.

## Scope

- Add explicit non-interactive behavior for policy-required questions.
- Map policy-required questions to a stop outcome without execution.
- Ensure CI/agent examples behave predictably.
- Avoid fallback to raw `npx`, npm, shell prompts, or package-manager behavior.

## Acceptance Criteria

- Non-interactive mode stops when any policy rule requires a question.
- Non-interactive policy stops use exit code `10` for ask-required.
- JSON output uses `decision: ask` or agreed stop representation with
  `required_next_action: ask_user`.
- Human output says the command did not run because interaction is required.
- Tests prove no package binary, lifecycle script, or dependency script runs in
  this path.

