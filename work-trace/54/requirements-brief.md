# Requirements Brief

- Add or wire a user-facing inspect-mode CLI path.
- Reuse M1 resolver and M2 extraction/no-execution primitives.
- Inspect mode must resolve, download, verify, extract evidence, render output,
  and exit without executing package code.
- Human and JSON output must flow from the same evidence model.
- Expected failures should use existing decision/reason vocabulary and must not
  panic.
- Tests must cover success, unsupported input, integrity failure, extraction
  failure, and M2 closure blocker cases.
