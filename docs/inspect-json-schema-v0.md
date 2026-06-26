# Inspect JSON Schema V0

`safe-npx --json` emits the M3 inspect schema for agents and CI. The schema is
an evidence contract, not a safety proof.

## Version

V0 uses:

```json
{
  "schema_version": "0.1"
}
```

## Top-Level Fields

The V0 top-level object contains:

- `schema_version`
- `artifact`
- `command_intent`
- `source_context`
- `authority_context`
- `facts`
- `heuristics`
- `external_evidence`
- `attestations`
- `release_diff`
- `decision_receipt`
- `decision`
- `reasons`
- `required_next_action`
- `execution`
- `exit_code`

`external_evidence`, `attestations`, and `release_diff` are reserved and null in
V0. M3 does not query a hosted audit registry, fetch third-party attestations,
or compute release-diff evidence for these fields. A non-null value requires a
later schema change with documented provenance and failure semantics. `execution`
is null for inspect mode.

`decision_receipt` is a non-authoritative inspect evidence record. It includes
artifact digest identity when available, command identity, evidence summary,
policy version, a nullable `issued_at` timestamp, and redaction metadata. In M3
it is not an approval, allow-list entry, or cache key; `cache_status` is
`not_an_approval_cache`, and later milestones must define validation, replay,
expiry, and cache semantics before receipts can affect execution.

During the `0.1` transition, JSON output also includes additive legacy fields so
existing callers can migrate without losing evidence paths:

- `package_spec`
- `recommendation`
- `status`
- `note`
- `inspect`
- `m1`

New agent and CI integrations should prefer the canonical V0 fields above.

## Compatibility

- Additive fields are allowed within `0.x`.
- Enum additions require a schema bump.
- Enum semantic changes require a migration note.

## Enum Vocabulary

`decision` values are:

- `allow`
- `ask`
- `deny`
- `unsupported`
- `inspection_error`
- `execution_refused`

`required_next_action` values are:

- `none`
- `ask_user`
- `retry_narrower_command`
- `inspect_only`
- `explicit_override`
- `unsupported`
