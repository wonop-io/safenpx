# Requirements Brief

Issue #9 acceptance criteria:

- Inspect mode leaves all sentinel files absent.
- Root package binary traps cannot run during inspection.
- Root lifecycle traps cannot run during inspection.
- Dependency lifecycle traps cannot run during inspection.
- Generated-shim traps cannot run during inspection.
- Network-attempt fixture is blocked or detected without executing package
  code.
- Harness runs in CI without relying on third-party package code.

Dependencies:

- #42 is closed and provides execution-closure decision/reason vocabulary.

Non-goals:

- Extract package tarballs.
- Execute package binaries, shims, or lifecycle scripts.
- Contact npm or any external network service.
- Decide the final M2 execution mechanism.
