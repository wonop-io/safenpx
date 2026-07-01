# Requirements Brief

## Source

- GitHub issue #73.
- M4 parent issue #6.
- Existing M3 inspect heuristic model and M4 policy engine.

## Scope

- Audit heuristic generation, policy evaluation, human output, and JSON output.
- Add guardrail tests that fail if heuristic-only evidence becomes a hard
  denial.
- Document that M4 heuristics are provisional risk signals, not safety proof.

## Acceptance Criteria

- Similar-name warnings do not produce `deny` by themselves.
- Unusual-shape warnings do not produce `deny` by themselves.
- Heuristics can contribute to `ask` only where explicitly documented.
- Reports label heuristics as provisional risk signals.
- Tests fail if a heuristic-only fixture becomes a hard denial.
