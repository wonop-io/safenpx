# Requirements Brief

## Problem

M3 should close only if every inspect evidence proof obligation has direct
evidence or an explicit later-milestone follow-up. The closeout must avoid
hand-wavy milestone completion.

## Acceptance Mapping

- Every M3 deliverable is complete or has an explicit follow-up.
- Every M3 acceptance criterion has direct evidence.
- `just test` passes.
- Latest GitHub Actions run on `main` passes.
- M3 has zero open implementation issues before closure.
- Closeout evidence is recorded in GitHub and `work-trace`.

## Constraints

- Do not close M3 if open M3 implementation issues remain.
- Do not create new product claims beyond the inspected evidence.
- Keep later-milestone work explicit rather than silently marking it complete.
