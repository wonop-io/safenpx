# Issue 31 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/31

## Request

Port the Wonop red-team / blue-team / judge review playbook into safe-npx, but
make it a prior-commit review and limit re-reviews to two.

## Scope

- Add a repository review-turn playbook.
- Adapt it to the issue-backed `work-trace/{issue-id}/` planning workflow.
- Require the review target to be an existing commit or commit range.
- Limit re-review loops to two.

## First Commit Rule

This trace scaffold is committed before implementation work begins.

