# Source Review Turn Context

Source repository:
`/Users/tfr/Documents/Projects/wonop/playbooks/repository/review-turn.md`

Relevant source behavior:

- Runs only when the user explicitly asks for an independent review pass.
- Requires clear acceptance criteria.
- Uses red-team, blue-team, and judge subagents.
- Subagents are read-only.
- Blocking P1/P2 findings must be resolved before delivery.

safe-npx adaptation:

- Review target must be a prior commit or explicit commit range.
- The review packet should include `work-trace/{issue-id}/` artifacts when
  available.
- Re-review loops are limited to two.

