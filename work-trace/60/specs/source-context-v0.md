# Source Context V0

## Premise

Inspect reports should tell users where the command claims to come from, but
safe-npx should not infer human intent from brittle process, terminal, or CI
signals. In M3, source context is caller-declared or unknown.

## Categories

The V0 source context vocabulary is:

- `manual_terminal`: a human says they are running the command from a terminal
- `docs_snippet`: a command copied from README, docs, blog, or tutorial text
- `agent_skill`: a coding agent, skill, or playbook says it is requesting the
  command
- `ci`: a CI job, workflow, or automation says it is requesting the command
- `unknown`: no trusted declaration was provided

## Input Behavior

V0 should use an explicit CLI flag. The default is `unknown`.

Invalid source-context values should fail closed at CLI parse time rather than
being coerced silently. This keeps mistakes visible and avoids a false sense of
authority about command origin.

## Report Placement

Source context belongs in the shared inspect model so human and JSON reports
consume the same field. It should be nested under authority context in M3, but
it must remain distinct from registry source, package scope, and future #12
redaction decisions.

## Fixture Coverage

Tests should cover all five categories plus invalid input behavior.
