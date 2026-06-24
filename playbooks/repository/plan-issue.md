---
title: Plan Issue
domain: repository
summary: Plan one small GitHub issue at a time with checked-in work-trace artifacts.
created: 2026-06-24
last_used: 2026-06-24
last_updated: 2026-06-24
---

# Plan Issue

Use this playbook before starting non-trivial work on a single GitHub issue.

The goal is to make the issue small, traceable, and handoff-ready before
implementation begins. Unlike the original Wonop `plan-work` skill, artifacts
live in the repository under `work-trace/{issue-id}/` and the initial trace
scaffold must be checked in before implementation work starts.

## Inputs

- One GitHub issue number.
- The user request or issue body.
- Relevant repository docs, code, policies, and prior decisions.

## Workflow

1. Confirm the issue exists and is the right issue for the requested work.
2. Create `work-trace/{issue-id}/` immediately.
3. Add the initial trace scaffold and commit it before implementation begins.
4. Capture the request, facts, assumptions, and evidence in `context/`.
5. Draft `requirements-brief.md` before solution design.
6. Split requirements and acceptance criteria into small tracked specs under
   `specs/`.
7. Keep `spec-index.md` and `progress.md` current as planning changes.
8. Update the GitHub issue with material scope, status, blocker, or acceptance
   changes.
9. Implement only after the trace scaffold has been checked in and the issue is
   small enough to execute safely.

## Required Work-Trace Layout

```text
work-trace/{issue-id}/
  README.md
  requirements-brief.md
  spec-index.md
  progress.md
  context/
  specs/
```

## Trace Rules

- Use the numeric GitHub issue id as `{issue-id}`.
- Keep trace files concise and reviewable.
- Record facts separately from assumptions.
- Do not store secrets, tokens, private keys, or raw credentials.
- Reuse the existing trace directory when continuing the same issue.
- If an issue becomes too large, split the GitHub issue before planning
  implementation.

## Minimum Scaffold Commit

The first commit for a planned issue should include at least:

- `work-trace/{issue-id}/README.md`
- `work-trace/{issue-id}/progress.md`

This commit proves the work is issue-backed before implementation begins.

## Verification

- The issue exists and is linked from the trace.
- `work-trace/{issue-id}/` is checked in before implementation files change.
- Requirements and acceptance criteria map to specs or explicit deferrals.
- `just test` passes before handoff.
