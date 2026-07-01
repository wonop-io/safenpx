# Required-Next-Action Spec

## Vocabulary

- `none`: no user or agent action is required before the current safe outcome.
- `ask_user`: stop and ask a human before execution can proceed.
- `retry_narrower_command`: retry with a narrower exact package command shape.
- `inspect_only`: use inspect evidence; execution is unavailable for this path.
- `explicit_override`: reserved semantic for a future override path.
- `unsupported`: stop because the requested path is outside current capability.

## Default Mapping

| Decision | Default next action |
| --- | --- |
| `allow` | `none` |
| `ask` | `ask_user` |
| `deny` | `none` |
| `unsupported` | `retry_narrower_command` |
| `inspection_error` | `inspect_only` |
| `execution_refused` | `inspect_only` |

## Reason-Specific Mapping

- Unsupported or malformed package specs: `retry_narrower_command`.
- Ambiguous or missing bins: `retry_narrower_command`.
- Lifecycle scripts, unsupported closure, and identity drift: `inspect_only`.
- Integrity mismatch: `none`.
- Registry, download, extraction, missing package, or missing version failures:
  `inspect_only`.
- Non-interactive ask-required stop: `ask_user`.

## Non-Goals

- No approval cache.
- No explicit override implementation.
- No new hosted evidence fields.

## Tests

- Policy-level tests cover default mappings and representative reason-specific
  mappings.
- JSON fixtures keep representative next actions stable.
- Human output includes a short safe-next-action explanation.

