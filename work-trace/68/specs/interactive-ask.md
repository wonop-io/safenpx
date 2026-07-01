# Interactive Ask Spec

## Current Baseline

#67 made threshold findings canonical policy evidence. Verified threshold cases
can already produce `PolicyDecision::Ask` and `PolicyNextAction::AskUser`.

## #68 Behavior

- Interactive ask is the human-facing interpretation of `decision=ask` and
  `required_next_action=ask_user`.
- Inspect mode still stops after evidence and decision rendering.
- Human output must show the policy reasons that caused the ask, not only the
  legacy M3 heuristic label.
- JSON output must carry `decision: "ask"`, `required_next_action:
  "ask_user"`, and policy findings for lifecycle, recent publish, and large
  package examples.

## Non-Goals

- No approval cache.
- No non-interactive fail-closed branch; #69 owns that.
- No final exit-code migration; #70 owns that.

## Tests

- Human inspect output for lifecycle evidence includes `lifecycle_script_present`
  and `ask_user`.
- JSON threshold fixtures cover lifecycle script, recent publish, and large
  package ask cases.
- Existing canary tests continue proving inspect mode does not execute package
  code.

