# Policy Evaluation Model

M4 introduces a policy layer between evidence gathering and output rendering.
The first slice should not implement all thresholds; it should make the decision
boundary explicit so later M4 tickets can add thresholds and fixtures without
duplicating logic.

## Model

- `PolicyEvaluation`
  - `policy_version`
  - `decision`
  - `reasons`
  - `required_next_action`
  - `rule_ids`

## Entrypoints

- M1/M3 report evidence to policy result.
- M2 closure reasons to policy result.

## Initial Rules

- Verified evidence follows the caller-requested recommendation for now.
- Unsupported and malformed inputs map to `unsupported`.
- Integrity mismatch maps to `deny`.
- Registry/missing package failures map to `inspection_error`.
- Unsupported closure maps to `execution_refused`.
- Ambiguous or missing bin maps to `unsupported`.
- Non-interactive stop preserves the legacy M2 `execution_refused` decision in
  #66 while carrying `ask_user` as the next action. Issues #69 and #70 own the
  M4 ask-required non-interactive stop and exit-code migration.
