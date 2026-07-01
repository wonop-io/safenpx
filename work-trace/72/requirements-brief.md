# Requirements Brief

## Source

- GitHub issue #72.
- M4 parent issue #6.
- Existing M4 policy, threshold, ask, exit-code, and next-action tickets.

## Scope

- Inventory current fixture and golden coverage for M4 rules.
- Add missing positive and negative fixtures where useful.
- Ensure every canonical decision and next action appears in deterministic
  test or golden coverage.
- Keep all fixtures local and non-executing.

## Acceptance Criteria

- Every policy rule has positive and negative coverage where applicable.
- Every canonical decision enum appears in fixture or integration coverage.
- Golden JSON remains deterministic and schema-compatible.
- Golden human output explains facts, heuristics, decision, reasons, and next
  action.
- Fixtures do not execute third-party package code.
