# Playbooks Index

Repository playbooks describe repeatable workflows for `safe-npx` work.

## Read Order

1. Load the root `index.md`.
2. Load this index.
3. Load the nearest domain index.
4. Load the smallest matching playbook.

## Repository

Start at `repository/index.md` for repository workflow playbooks.

- `repository/plan-issue.md`: Plan one small GitHub issue at a time and check in
  its `work-trace/{issue-id}/` scaffold before implementation begins.
- `repository/playbook-specification.md`: Define required playbook metadata,
  indexing, and validation rules.
- `repository/review-turn.md`: Run a prior-commit red-team, blue-team, judge
  review with at most two re-reviews.
