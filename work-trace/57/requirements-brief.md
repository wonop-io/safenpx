# Requirements Brief

Issue #57 acceptance criteria:

- Evidence model has distinct fields for facts, heuristics, decision, reasons,
  required next action, authority context, and execution state.
- Heuristics are report-only in M3 and do not become hard denials before M4
  policy work.
- Dependency declarations cannot be serialized as verified dependency artifacts
  unless explicitly marked verified by a later closure proof.
- Human and JSON renderers consume the same evidence model.
- Tests cover fact-only evidence, heuristic evidence, unsupported/refusal
  evidence, and missing optional facts.

Dependencies:

- #56 provides registry metadata evidence.
- #11 provides verified root package evidence.

Non-goals:

- Full authority redaction rules, owned by #12.
- Caller-declared source context depth, owned by #60.
- Final human report polish, owned by #58.
- JSON schema compatibility and golden fixtures, owned by #5 and #59.
- M4 hard policy denials based on heuristics.
