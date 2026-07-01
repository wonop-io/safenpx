# Requirements Brief

## Source

GitHub issue #75: M4: Document policy matrix and exit behavior.

## Acceptance Criteria

- Documentation includes the complete exit-code table.
- Documentation includes the M4 threshold table and provisional status.
- Documentation includes JSON examples or links to golden fixtures for each
  representative outcome.
- Documentation warns that heuristics are signals, not proof of safety.
- Docs are linked from the roadmap or milestones narrative.

## Constraints

- Keep docs durable and implementation-aligned.
- Prefer linking existing golden fixtures over duplicating large JSON payloads.
- State M4 limits clearly: no dependency closure proof, no package-manager
  integration, no assertion that a package is safe.
