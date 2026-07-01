# Requirements Brief

## Source

GitHub issue #76: M4: Close out policy v0 milestone.

## Acceptance Criteria

- All M4 implementation issues are closed or explicitly deferred with rationale.
- `just test` passes locally.
- GitHub Actions is green on the closeout commit.
- Closeout evidence records commit, CI run, issue state, milestone state, and
  residual risks.
- M4 milestone is closed only after zero open M4 issues remain.

## Constraints

- Do not close the parent M4 issue until child issue state and CI state are
  verified.
- Record residual risk honestly rather than laundering it into success.
- Keep the closeout artifact short enough to review and commit.
