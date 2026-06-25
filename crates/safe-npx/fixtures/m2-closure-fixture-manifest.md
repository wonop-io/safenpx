# M2 Closure Fixture Manifest

The canonical machine-readable fixture file is
`m2-closure-fixture-manifest.txt`.

Each row has:

```text
id|kind|description|expected_decision|expected_reason|expected_exit_code|sentinel
```

## Required Kinds

The manifest must include at least one row for each M2 closure surface:

- `canary`
- `bin`
- `lifecycle`
- `dependency`
- `registry`
- `race`
- `cache`
- `shim`
- `closure`

## Adding A Closure Trap

Add trap metadata only. Do not add shell scripts, package-manager invocations,
or fixture payloads that execute during tests.

For each new row:

- Use a stable lowercase id.
- Pick the narrowest fixture kind.
- Set the expected decision and reason from the M2 closure vocabulary.
- Set the expected exit code that callers should eventually expose.
- Use `no_execution` unless a test consumes a more specific sentinel.
- Add or update a test that consumes the row when the subsystem exists.

Canary rows must correspond to entries in `canary-fixture-manifest.txt`.
