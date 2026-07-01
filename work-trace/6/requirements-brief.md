# Requirements Brief

## User Request

Plan M4 carefully and create a comprehensive set of tickets, each with
acceptance criteria.

## Facts

- M4 is `Provisional Policy V0 And Exit Semantics`.
- Existing parent issue: #6, `M4: Implement provisional policy v0 and exit
  semantics`.
- M4 must turn M3 inspect evidence into predictable decisions and exit codes.
- The milestone scope includes interactive and non-interactive modes, policy
  thresholds, exit codes, fixture coverage, and agent-readable JSON semantics.
- M5 depends on M4 for trustworthy execute-mode and approval-cache behavior.

## Assumptions

- Issue #6 should become the M4 parent/index rather than the only implementation
  ticket.
- M4 should avoid adding execution override behavior beyond decision semantics;
  actual broader execute behavior belongs in M5.
- Similar-name and unusual-shape heuristics must remain report-only in M4.

## Non-Goals

- Hosted audits, release diffs, attestations, and approval caching.
- Broad dependency-closure execution.
- Changing M2's direct-extract execution decision.
