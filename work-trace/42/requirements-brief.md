# Requirements Brief

Issue #42 acceptance criteria:

- Contracts distinguish declared dependencies from verified dependency
  artifacts.
- Contracts can represent root artifact identity, selected bin identity,
  generated shim identity, and lifecycle-script presence.
- M2 reasons serialize with stable snake_case names.
- Unit tests cover serialization and refusal-state mapping.
- No contract implies that root tarball verification alone is sufficient to
  execute.

Non-goals:

- Extract package contents.
- Select bins.
- Run package code.
- Implement execution.
- Build the canary harness.
