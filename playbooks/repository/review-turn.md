---
title: Review Turn
domain: repository
summary: Run a prior-commit red-team, blue-team, judge review with at most two re-reviews.
created: 2026-06-24
last_used: null
last_updated: 2026-06-24
---

# Review Turn

Use this playbook when the user explicitly asks for an independent review pass
over completed work.

This is a prior-commit review. The review target must be an existing commit or
an explicit commit range, not uncommitted working-tree changes.

Run the red-team, blue-team, judge review only when all conditions are true:

- the user explicitly asks for review
- a GitHub issue exists for the work
- clear acceptance criteria exist in the issue, user request, or accepted spec
- the reviewed work is committed
- required verification for the work has already run

Do not use this playbook as a substitute for tests, policy checks, or normal
implementation review.

## Goal

Use independent read-only review roles to test a completed prior commit against
accepted criteria:

- Red team attacks the commit.
- Blue team defends or concedes each attack with evidence.
- Judge decides which attacks are findings of record.

The result should make blocking defects impossible to miss while keeping
non-blocking residual risk visible.

## Required Inputs

Before starting, collect:

1. GitHub issue number and URL.
2. User request and accepted scope.
3. Acceptance criteria and their source.
4. Commit SHA or commit range under review.
5. `work-trace/{issue-id}/` artifacts, if present.
6. Changed files and important diff context.
7. Verification commands and outcomes.
8. Known skipped checks, assumptions, or risk areas.

If the review target is not committed, stop and ask for a commit first. If the
acceptance criteria are missing or too vague, ask the user to provide or approve
criteria before running the review.

## Priority Scale

Use these labels exactly:

- P1: Critical correctness, safety, data-loss, security, build, or test failure
  that blocks delivery.
- P2: Unmet acceptance criterion, likely regression, policy violation, missing
  required behavior, or materially incomplete fix that blocks delivery.
- P3: Non-blocking bug risk, test gap, edge case, or maintainability issue tied
  to accepted criteria, direct regressions, or delivery blockers.
- P4: Minor quality, documentation, naming, or ergonomics issue tied to accepted
  criteria, direct regressions, or delivery blockers.
- P5: Low-priority question or observation about accepted criteria, direct
  regressions, delivery blockers, or skipped verification.

## Operator Flow

### 1. Build The Review Packet

Summarize for all review agents:

- issue and accepted scope
- acceptance criteria
- commit SHA or commit range
- changed files and key diff context
- relevant `work-trace/{issue-id}/` artifacts
- verification commands and outcomes
- known risk areas and skipped checks

Instruct every review agent that it is read-only and must not edit files.
Findings must trace to an acceptance criterion, direct regression, or delivery
blocker.

### 2. Red Team

Ask a read-only red team reviewer to attack the prior commit against the
acceptance criteria.

The red team must:

- inspect adversarial inputs, edge cases, missing behavior, incorrect
  assumptions, regressions, and ways the commit can fail the criteria
- tie every attack to an acceptance criterion, direct regression, or delivery
  blocker
- include file paths and line numbers when possible
- avoid fixes, style preferences, or optional improvements unless they affect a
  criterion or blocker
- say `No attacks` when it finds nothing plausible

### 3. Blue Team

Ask a second read-only reviewer to defend the prior commit against the red
team's attacks.

The blue team receives the review packet and red team output. It must:

- answer every red team attack
- prove with code, docs, tests, or command evidence that the attack is already
  handled, or concede that it lands
- include file paths, line numbers, command outcomes, or log excerpts when they
  support the defense
- avoid generic reassurance and unrelated improvements

### 4. Judge

Ask a third read-only reviewer to decide which attacks are findings.

The judge receives the review packet, red team output, and blue team output. It
must:

- decide whether each attack is valid, invalid, or uncertain
- label every valid or uncertain issue P1, P2, P3, P4, or P5
- tie every issue to the relevant acceptance criterion, direct regression, or
  delivery blocker
- include file paths and line numbers when possible
- say `No findings` when nothing actionable remains

### 5. Resolve Blocking Findings

For P1 and P2 findings:

- fix the issue, or determine with evidence that it is invalid
- commit the fix
- rerun required verification
- run a re-review only if the fix materially changes the reviewed surface

Re-reviews are capped at two total rounds after the initial review. If valid P1
or P2 findings remain after two re-reviews, stop and report the remaining
blocker instead of looping indefinitely.

### 6. Report Results

In the final handoff or issue comment, include:

- commit or commit range reviewed
- verification status
- P1/P2 resolution summary
- remaining P3-P5 findings, if any
- explicit `no remaining P3-P5 findings` when none remain
- re-review count used

## Quality Bar

A good review turn:

- reviews committed work, not a dirty working tree
- starts from a GitHub issue and clear acceptance criteria
- includes `work-trace/{issue-id}/` artifacts when available
- uses separate red-team, blue-team, and judge roles
- keeps review agents read-only
- prioritizes concrete defects over style preferences
- caps re-review loops at two

## Anti-Patterns

Avoid:

- reviewing uncommitted changes
- inventing acceptance criteria to make the review run
- broadening the review beyond accepted criteria, regressions, or blockers
- skipping the judge and treating red-team attacks as findings of record
- using review as a replacement for required verification
- letting review agents edit files
- hiding unresolved P3-P5 findings
- running more than two re-review rounds

