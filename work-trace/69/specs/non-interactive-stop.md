# Non-Interactive Stop Spec

## Behavior

- A policy result with `decision=ask` and `required_next_action=ask_user`
  requires interaction.
- In non-interactive contexts, safe-npx must stop before execution and return
  exit code `10`.
- JSON keeps `decision: "ask"` and `required_next_action: "ask_user"` so agents
  see that a human question is required.
- Human output must say the command did not run because interaction is required.

## Mode Detection

Initial implementation should prefer an explicit CLI flag or existing
source-context classification over terminal probing. Source contexts for agents
and CI are non-interactive by definition.

## Non-Goals

- No approval cache.
- No package execution.
- No raw npm/npx fallback.

## Tests

- CI/agent context plus lifecycle ask returns exit code `10`.
- JSON output includes `decision: "ask"` and `required_next_action: "ask_user"`.
- Human output includes a clear non-execution interaction-required statement.
- Existing canary/no-execution tests remain green.

