# Issue 48 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/48

## Request

Seed the M2 closure fixture manifest and golden outcomes so later execution
closure work cannot hand-wave missing trap coverage.

## Scope

- Add a machine-readable M2 closure fixture manifest.
- Cover fixture kinds for canary, bin, lifecycle, dependency, registry, race,
  cache, shim, and closure cases.
- Include expected decision, reason, exit code, and no-execution sentinel in
  every row.
- Add tests that consume the manifest and fail on missing fixture kinds.
- Add fixture docs explaining how to add closure traps safely.

## First Commit Rule

This trace scaffold is committed before implementation begins.
