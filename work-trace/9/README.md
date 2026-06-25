# Issue 9 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/9

## Request

Build a reusable no-package-code-ran canary harness for M2 inspection and
execution-closure tests.

## Scope

- Model local trap fixtures for package binaries, lifecycle scripts, dependency
  lifecycle scripts, generated shims, and network attempts.
- Prove inspect-mode behavior leaves every sentinel untouched.
- Keep fixtures deterministic, local, and suitable for CI.
- Expose the harness for later closure and execution-spike tests.

## First Commit Rule

This trace scaffold is committed before implementation begins.
