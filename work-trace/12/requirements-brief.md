# Requirements Brief

Issue #12 acceptance criteria:

- Reports never print secret values, private registry tokens, raw auth headers,
  or full environment dumps.
- JSON avoids home paths and machine-specific full paths unless explicitly
  redacted or categorized.
- Redacted display output is separate from canonicalized identity fields used
  for receipts/cache keys.
- Authority context examples include local terminal, CI, trusted project
  directory, temp directory, public npm, scoped registry, manual user, and
  coding agent.
- Human report text states authority context is not sandboxing.
- Golden fixtures cover token redaction, home-path redaction, scoped registry
  display, temp directory classification, and CI/agent context categories.

Dependencies:

- #60 provides caller-declared source context.

Non-goals:

- Full decision receipt schema, owned by #61.
- Final JSON schema compatibility, owned by #5.
- Golden fixture file corpus, owned by #59; this issue can add focused
  deterministic tests that become fixture seeds.
- Actual sandboxing or process isolation.
