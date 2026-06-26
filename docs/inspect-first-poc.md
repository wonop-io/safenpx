# Inspect-First POC

M3 makes `safe-npx` useful before broad execution support exists. It resolves,
downloads, verifies, inspects, reports, and stops.

The point is to replace a blind package-execution prompt with evidence that a
human or coding agent can read before deciding what should happen next.

## Human Workflow

Use the explicit inspect action:

```bash
safe-npx inspect create-example@1.2.3
```

The human report separates:

- command intent,
- verified package facts,
- decision and required next action,
- authority context,
- execution state,
- provisional heuristics,
- safety boundary.

Inspect mode is no-run by design. A successful inspect report means package
evidence was collected. It does not mean the package is safe, endorsed,
audited, or approved for execution.

## Agent And CI Workflow

Use JSON output for deterministic agent and CI consumption:

```bash
safe-npx --json inspect create-example@1.2.3
```

The M3 JSON schema is documented in `docs/inspect-json-schema-v0.md`. New
integrations should consume the canonical V0 fields:

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

Historical forms such as `safe-npx --json create-example@1.2.3` are retained
during the `0.1` transition, but M3 documentation and future examples should
prefer the explicit `inspect` action.

## Supported Command Shapes

M3 supports one package spec per command:

- `safe-npx inspect name@version`
- `safe-npx inspect @scope/name@version`
- forwarded package arguments after `--`, for example
  `safe-npx inspect create-example@1.2.3 -- --template react`

The command shape identifies the package to inspect and preserves forwarded
arguments in command intent. Forwarded arguments are evidence only in M3; they
are not executed.

## Unsupported Command Shapes

M3 rejects or classifies these shapes instead of falling back to raw `npx`:

- unversioned names such as `safe-npx inspect create-example`,
- floating tags such as `safe-npx inspect create-example@latest` until
  tag-race proof is implemented,
- version ranges other than `latest`,
- Git URLs,
- local paths,
- tarball URLs,
- npm aliases,
- multiple package specs,
- `npm exec --package`,
- `npm exec -c`,
- raw `npm`, `npx`, or `npm-exec` command variants passed through `safe-npx`,
- package-manager-specific execution variants.

Malformed specs should not cause network calls. Unsupported specs should state
what was rejected and whether anything was downloaded.

## What M3 Catches

M3 can report evidence such as:

- exact resolved package identity,
- registry source,
- integrity verification result,
- publish time,
- publisher and maintainer fields when available,
- repository, license, and provenance fields when available,
- package size and file count,
- binary declarations,
- lifecycle script declarations,
- dependency declarations,
- root package metadata shape,
- provisional heuristics such as lifecycle-script presence or unusual package
  shape.

Dependency declarations are only declarations in M3 unless a later milestone
proves and verifies the full dependency execution closure.

## What M3 Does Not Catch

M3 does not prove a package is safe. It does not:

- run dynamic malware analysis,
- execute package binaries,
- execute lifecycle scripts,
- install or execute dependency trees,
- prove maintainer intent,
- prove that package source matches repository source,
- query a hosted audit registry,
- verify third-party attestations,
- compute release-diff evidence,
- sandbox future execution,
- authorize package execution.

The absence of a warning is not approval. Heuristics are decision support, not
proof.

## Redaction And Authority Context

Reports and JSON include authority context so users can reason about ambient
authority before execution. M3 records categories such as:

- source context: `manual_terminal`, `docs_snippet`, `agent_skill`, `ci`, or
  `unknown`,
- cwd trust class,
- registry authority,
- package scope,
- command intent.

Authority context is not a sandbox. It describes what kind of environment is
asking to inspect a package.

Display output is redacted. Reports should not print secret values, private
registry tokens, full sensitive environment details, or home-directory paths.
Canonical or hashed identity fields used for future receipts stay separate from
redacted display values.

## Schema Compatibility

The M3 JSON contract uses `schema_version: "0.1"`.

Compatibility rules:

- additive fields are allowed within `0.x`,
- enum additions require a schema bump,
- enum semantic changes require a migration note.

The reserved fields `external_evidence`, `attestations`, and `release_diff` are
present and null in V0. A non-null value must come with a later schema change
that documents provenance and failure semantics.

`decision_receipt` is present as a non-authoritative inspect evidence record. In
M3 it is not an approval, allow-list entry, approval cache key, or execution
permission.

## Execution Boundary

M3 does not broadly execute packages. The `execution` JSON field is null for
inspect mode, and the human report must state that package code did not run.

Execution-mode support belongs to later milestones and remains constrained by
the M2 decision: only a verified direct-extract subset may execute, and anything
outside the proven closure must return `execution_refused` or remain
inspect-only.
