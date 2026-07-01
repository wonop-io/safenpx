# Provisional Thresholds Spec

## Policy Shape

M4 policy keeps a single canonical policy evaluation result. #67 extends it with
threshold findings: stable rule id, stable reason, observed value, threshold
value, and a short evidence string.

## Threshold Constants

- `M4_RECENT_PUBLISH_WARNING_HOURS = 24`
- `M4_LARGE_TARBALL_WARNING_BYTES = 5 * 1024 * 1024`
- `M4_LARGE_FILE_COUNT_WARNING = 500`

The constants live with policy evaluation so code and tests share one source of
truth. `docs/milestones.md` remains the human-readable roadmap table.

## Decision Semantics

- Verified evidence with no threshold findings keeps the caller recommendation.
- Threshold warnings are heuristic warnings. If caller recommendation is
  `allow`, warnings escalate the policy decision to `ask` with `ask_user`.
- If the caller recommendation is already `ask`, threshold findings add reasons
  and rule ids without changing the required action.
- If the caller recommendation is `deny`, threshold findings do not weaken the
  denial.
- Integrity mismatch and other proof failures stay separate hard-denial paths.

## Evidence Inputs

- `registry_evidence.publish_time` supplies recent publish evidence.
- `static_extraction.artifact_size_bytes` supplies tarball size evidence.
- `static_extraction.file_count` supplies file-count evidence.
- `static_extraction.metadata.lifecycle_scripts` supplies lifecycle evidence.

## Tests

- Direct policy tests cover each threshold.
- Tests assert decisions, reasons, rule ids, and evidence values.
- A hard-deny caller recommendation remains denied when warnings are present.

