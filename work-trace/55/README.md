# Issue 55 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/55

## Request

Carry the reusable no-package-code-ran canary harness from M2 into the actual
M3 inspect-mode pipeline.

## Scope

- Exercise the real `safe-npx inspect <exact-spec>` report path, not only the
  lower-level metadata inspection helpers.
- Reuse the existing local canary fixture model for root binaries, lifecycle
  scripts, dependency lifecycle scripts, generated shims, and network attempts.
- Cover human and JSON rendering so report formatting cannot accidentally
  trigger another evidence pass or package-code execution path.
- Keep all fixtures local, deterministic, and CI-safe.

## First Commit Rule

This trace scaffold is committed before implementation begins.
