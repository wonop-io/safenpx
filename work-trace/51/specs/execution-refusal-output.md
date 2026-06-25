# M2 Execution Refusal Output

M2 refusal output is the user-visible stop sign for unproven execution
closures. When inspection succeeds but execution cannot be proven safe enough
for the current milestone, the report must return `execution_refused` rather
than a generic inspection error.

The human report must:

- state that execution was refused,
- list the stable reason names that caused the refusal,
- avoid raw `npx` or package-manager fallback guidance.

The JSON report must expose stable machine-readable fields for:

- `decision`,
- `reasons`,
- `required_next_action`,
- `execution`,
- `exit_code`.

M2 refusal tests must cover each reason introduced by the completed proof
tickets: unsupported closure, ambiguous bin, missing bin, lifecycle script,
registry precedence mismatch, cache identity mismatch, shim identity mismatch,
and non-interactive stop.

