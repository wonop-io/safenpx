# M4 Policy V0 Reference

M4 turns inspect evidence into stable local decisions, next actions, and exit
codes. This policy is provisional: thresholds are intentionally conservative
constants until fixture coverage and dogfooding show better values.

Policy output is evidence for a decision. It is not proof that a package is
safe.

## Decisions

| Decision | Meaning |
| --- | --- |
| `allow` | Evidence and policy permit the requested stopped report. |
| `ask` | A human decision is required before execution can proceed. |
| `deny` | A proof failure or known unsafe condition was found. |
| `unsupported` | The requested input or command shape is outside current capability. |
| `inspection_error` | Evidence could not be collected reliably. |
| `execution_refused` | Inspection succeeded, but execution closure cannot be proven. |

## Exit Codes

| Code | Decision path | Meaning |
| --- | --- | --- |
| `0` | `allow` or interactive `ask` | Inspection completed and no non-interactive stop was required. |
| `10` | non-interactive `ask` | Policy needs a human question, but the source context cannot answer a prompt. |
| `11` | `deny` | A proof failure or hard stop was found. |
| `12` | `unsupported` | Retry with a narrower supported exact package command. |
| `13` | `inspection_error` | Registry, download, extraction, or evidence collection failed. |
| `14` | `execution_refused` | Execution would require an unproven closure. |
| `15` | delegated execution failure | Reserved for a future delegated execution path. |

## Required Next Actions

| Action | Meaning |
| --- | --- |
| `none` | No follow-up action is required for the stopped report. |
| `ask_user` | Ask a human before execution can proceed. |
| `retry_narrower_command` | Retry with a narrower exact command shape. |
| `inspect_only` | Use inspect evidence; execution is unavailable here. |
| `explicit_override` | Reserved for a future explicit override path. |
| `unsupported` | Stop; this path is outside current capability. |

## Current Reasons

M4 currently emits these stable reason names:

- `caller_requested_allow`
- `caller_requested_ask`
- `caller_requested_deny`
- `unsupported_spec`
- `malformed_spec`
- `registry_error`
- `integrity_mismatch`
- `missing_package`
- `missing_version`
- `ambiguous_bin`
- `missing_bin`
- `lifecycle_script_present`
- `recent_publish`
- `large_package`
- `large_file_count`
- `unsupported_closure`
- `metadata_changed`
- `cache_identity_mismatch`
- `registry_precedence_mismatch`
- `shim_identity_mismatch`
- `non_interactive_stop`

## Provisional Thresholds

| Rule | Threshold | Effect |
| --- | --- | --- |
| Recent publish | Published less than `24` hours ago | Adds `recent_publish`; escalates `allow` to `ask`. |
| Large package | Tarball larger than `5 * 1024 * 1024` bytes | Adds `large_package`; escalates `allow` to `ask`. |
| Large file count | More than `500` files | Adds `large_file_count`; escalates `allow` to `ask`. |
| Lifecycle script | Any root package lifecycle script | Adds `lifecycle_script_present`; escalates `allow` to `ask`. |
| Integrity mismatch | Downloaded bytes do not match registry integrity | Returns `deny`. |
| Unsupported input | Unsupported or malformed command shape | Returns `unsupported`. |
| Evidence failure | Registry, version, or extraction evidence unavailable | Returns `inspection_error`. |
| Unproven execution closure | Execution cannot prove the same bytes will run | Returns `execution_refused`. |

Threshold findings are warnings that affect the local decision. They are not
malware verdicts. Similar-name and unusual-shape heuristics remain report-only
signals until validated by later fixtures and policy work.

## Representative Outcomes

| Outcome | Example evidence |
| --- | --- |
| `allow` | Inline example below; covered by CLI integration tests in `crates/safe-npx/src/decision_integration_tests.rs`. |
| interactive `ask` | [`inspect-json-schema-v0-ask.json`](../crates/safe-npx/fixtures/inspect-json-schema-v0-ask.json) |
| non-interactive `ask` | [`inspect-json-schema-v0-redacted-authority.json`](../crates/safe-npx/fixtures/inspect-json-schema-v0-redacted-authority.json) |
| `deny` | [`inspect-json-schema-v0-integrity-failure.json`](../crates/safe-npx/fixtures/inspect-json-schema-v0-integrity-failure.json) |
| `unsupported` | [`inspect-json-schema-v0-unsupported.json`](../crates/safe-npx/fixtures/inspect-json-schema-v0-unsupported.json) |
| `inspection_error` | [`inspect-json-schema-v0-failure.json`](../crates/safe-npx/fixtures/inspect-json-schema-v0-failure.json) |
| lifecycle ask | [`inspect-json-schema-v0-static-blockers.json`](../crates/safe-npx/fixtures/inspect-json-schema-v0-static-blockers.json) |

Allow is represented by the same JSON envelope and is covered by CLI
integration tests. The minimal shape is:

```json
{
  "decision": "allow",
  "reasons": ["caller_requested_allow"],
  "required_next_action": "none",
  "execution": null,
  "exit_code": 0
}
```

Execution refusal is also covered by CLI integration tests. The minimal shape
is:

```json
{
  "decision": "execution_refused",
  "reasons": ["unsupported_closure"],
  "required_next_action": "inspect_only",
  "execution": null,
  "exit_code": 14
}
```

Non-interactive stop is an ask-required stop, not a denial:

```json
{
  "decision": "ask",
  "reasons": ["caller_requested_allow", "lifecycle_script_present"],
  "required_next_action": "ask_user",
  "execution": null,
  "exit_code": 10
}
```

## What M4 Does Not Prove

M4 does not prove that a package is safe. It does not verify dependency closure,
query a hosted audit registry, prove maintainer intent, prove package-manager
delegation, or run package code. Unsupported or unverifiable paths must fail
closed rather than falling back to raw `npx`.

Heuristics are provisional risk signals. They help decide whether to ask, deny,
retry, or inspect only; they are not malware labels.
