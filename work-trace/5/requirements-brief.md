# Requirements Brief

Issue #5 acceptance criteria:

- JSON includes `schema_version` and all M3 top-level fields from
  `docs/milestones.md`.
- JSON enum values match the M3 milestone vocabulary exactly.
- `execution` is always null for inspect-mode output.
- Nullable reserved fields are present and null in V0.
- Agents and CI can make decisions from JSON without parsing human terminal
  output.
- Schema compatibility rules are documented and enforced by tests or golden
  fixtures.
- Golden fixtures verify deterministic output for at least one ask-style
  inspection, one unsupported input, and one inspection/refusal failure.

Dependencies:

- M3 inspect pipeline ticket.
- M3 evidence model ticket.
- #12 authority-context redaction.

Non-goals:

- Hosted audits, attestations, and release diffs.
- Execute-mode JSON population.
- Full golden fixture corpus owned by #59, beyond the minimum needed here.
