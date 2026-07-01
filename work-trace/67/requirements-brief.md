# Requirements Brief

## Source

- GitHub issue #67.
- `docs/milestones.md` M4 threshold table.
- Existing M4 policy model from issue #66.

## Required Thresholds

- Recent publish warning: package version published within 24 hours.
- Large package warning: verified tarball larger than 5 MB.
- Large file-count warning: verified artifact contains more than 500 files.
- Lifecycle script: prompt/ask in interactive mode; non-interactive stop is
  owned by later M4 exit semantics, but #67 must expose a stable lifecycle rule
  signal and policy reason.

## Acceptance Criteria

- Each threshold has a stable rule id.
- Threshold constants are defined once and referenced by tests/docs.
- Recent publish, large package, large file count, and lifecycle cases produce
  expected decisions/reasons.
- Heuristic warnings escalate allow to ask but do not become hard denials.
- Threshold output includes human/agent-readable evidence describing the trigger.

