# M2 Closure Fixture Manifest Spec

## Manifest Shape

The seed manifest should use a stable pipe-delimited row format:

```text
id|kind|description|expected_decision|expected_reason|expected_exit_code|sentinel
```

Each row represents a future or current M2 proof case. Seed rows may be
contract-only, but they must still carry golden outcomes that later
implementation tests consume.

## Required Kinds

- `canary`
- `bin`
- `lifecycle`
- `dependency`
- `registry`
- `race`
- `cache`
- `shim`
- `closure`

## Sentinel Vocabulary

Use `no_execution` when the fixture must prove no package code ran. Use more
specific future sentinel names only when a test consumes them.

## Safety Rule

Adding a trap fixture means adding metadata that describes the trap, not adding
shell scripts or package-manager commands that execute during tests.
