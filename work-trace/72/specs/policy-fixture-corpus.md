# Policy Fixture Corpus

## Required Coverage

- Thresholds: recent publish, large tarball, large file count, lifecycle
  scripts.
- Resolver and parser stops: integrity mismatch, resolver ambiguity,
  unsupported input, unsupported closure, inspection error.
- Context stops: non-interactive ask.
- Decisions: allow, ask, deny, unsupported, inspection_error,
  execution_refused.
- Next actions: none, ask_user, retry_narrower_command, inspect_only,
  explicit_override, unsupported.

## Fixture Rules

- Use local deterministic tarballs or structured model fixtures.
- Do not run package code.
- Keep human and JSON goldens stable and redacted.
- Prefer targeted policy tests when a full tarball fixture adds no evidence.

## Coverage Inventory

- Positive threshold coverage: recent publish, large tarball, large file count,
  and lifecycle scripts in `policy_tests`.
- Negative threshold coverage: old publish time, exact size boundary, exact
  file-count boundary, no lifecycle scripts, and clean allow evidence in
  `policy_tests`.
- Fail-closed M1 coverage: unsupported input, malformed input, integrity
  mismatch, registry error, missing package, and missing version.
- Fail-closed M2 coverage: resolver ambiguity, missing bin, unsupported
  closure, lifecycle script, identity drift, shim drift, and non-interactive
  stop through policy, report, and fixture tests.
- Golden coverage: representative human outputs for normal ask, static
  blockers, unsupported input, integrity failure, redaction, and missing
  optional metadata; JSON schema goldens for ask, unsupported,
  inspection-error, integrity-deny, static blockers, and redaction.
- Reserved next actions: `explicit_override` and `unsupported` remain vocabulary
  compatibility entries only until a later milestone introduces producing
  rules.
