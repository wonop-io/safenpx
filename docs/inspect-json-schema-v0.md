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
- `policy`
- `execution`
- `exit_code`

`external_evidence`, `attestations`, and `release_diff` are reserved and null in
V0. M3 does not query a hosted audit registry, fetch third-party attestations,
or compute release-diff evidence for these fields. A non-null value requires a
later schema change with documented provenance and failure semantics. `execution`
is null for inspect mode.

`policy` contains the canonical M4 policy evaluation used to derive
agent-facing `decision`, `reasons`, and `required_next_action` during the M4
transition. It includes `policy_version`, stable `reasons`, stable `rule_ids`,
and provisional `findings` with observed values and thresholds when a policy
threshold fires. Integrations should treat `policy.findings` as evidence for
the decision, not as proof that a package is safe.

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

Reserved fields are part of compatibility. `external_evidence`, `attestations`,
and `release_diff` must remain present and null until a later milestone defines
non-null provenance, trust, freshness, and failure semantics.

## Redaction And Authority Context

`authority_context` describes the ambient authority around an inspect request.
It includes categories such as source context, cwd trust class, registry
authority, package scope, and command intent. It is not a sandbox and should not
be interpreted as proof that future execution is isolated.

JSON display fields are redacted. Reports must not expose secret values, private
registry tokens, full sensitive environment details, or home-directory paths.
Canonical or hashed identity fields used for future receipts are separate from
redacted display strings.

## Execution Boundary

M3 inspect JSON is no-run evidence. `execution` is null for inspect mode, and a
successful report means inspection completed without running package binaries,
lifecycle scripts, dependency scripts, or raw `npx`.

Evidence signals are not safety proofs. They help a human or agent decide what
to do next; they do not authorize execution.

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
