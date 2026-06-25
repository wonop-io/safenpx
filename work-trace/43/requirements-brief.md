# Requirements Brief

Issue #43 acceptance criteria:

- Extraction rejects path traversal entries.
- Extraction rejects or safely records symlink/hardlink entries that could
  escape the inspection root.
- Extraction never invokes lifecycle scripts, package binaries, shell commands,
  or package managers.
- Extracted package metadata is tied to the verified artifact digest from M1.
- Tests cover normal package, traversal attempt, symlink escape attempt, missing
  `package.json`, and malformed `package.json`.

Dependencies:

- #42 is closed and provides M2 closure vocabulary.
- #9 is closed and provides reusable no-package-code-ran canary fixtures.

Non-goals:

- Bin selection.
- Lifecycle/dependency policy decisions.
- Full dependency closure extraction.
- Execution or package-manager delegation.
