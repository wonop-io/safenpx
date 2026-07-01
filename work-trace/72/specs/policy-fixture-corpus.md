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
