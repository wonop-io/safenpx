# Requirements Brief

Issue #60 acceptance criteria:

- Source context enum includes manual terminal, docs snippet, agent skill, CI,
  and unknown.
- V0 source context is caller-declared or unknown, not inferred from brittle
  heuristics.
- Human and JSON reports include source context.
- Invalid source-context input fails closed or falls back to unknown according
  to documented rules.
- Fixtures cover each source context category.

Dependencies:

- #57 provides the shared inspect evidence model.

Non-goals:

- Authority-context redaction policy, owned by #12.
- JSON schema compatibility guarantees, owned by #5.
- Golden fixture corpus, owned by #59.
- Automatic detection of calling tools, terminals, CI vendors, or agent names.
