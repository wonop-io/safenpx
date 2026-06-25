# Requirements Brief

Issue #48 acceptance criteria:

- Manifest includes fixture kinds for canary, bin, lifecycle, dependency,
  registry, race, cache, shim, and closure.
- Every fixture row has expected decision, reason, exit code, and sentinel
  behavior.
- Tests consume the manifest rather than leaving it as dead documentation.
- Missing fixture kinds fail tests with an actionable message.
- Fixture docs explain how to add a new closure trap safely.

Dependencies:

- #42 is closed and provides M2 closure decision/reason vocabulary.
- #9 is closed and provides no-package-code-ran canary fixtures.

Non-goals:

- Implement bin selection.
- Implement lifecycle/dependency blockers.
- Implement registry/race/cache/shim behavior.
- Execute package code.
