# Requirements Brief

## Source

GitHub issue #74: M4: Add decision enum integration coverage.

## Acceptance Criteria

- Each decision enum value has at least one CLI-level test.
- Tests assert decision, reasons, required next action, and exit code together.
- Human and JSON renderers agree on the same underlying policy result.
- Non-interactive ask-required behavior is covered at CLI level.
- No test weakens no-package-code-ran invariants.

## Constraints

- Keep package fixture execution inert.
- Preserve canary protections that prove lifecycle scripts did not run.
- Prefer existing fixture and assertion helpers over new harness machinery.
- Keep new coverage scoped to M4 decision and exit semantics.
