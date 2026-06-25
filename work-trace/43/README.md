# Issue 43 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/43

## Request

Extract the verified root artifact into a controlled inspection workspace so
later M2 checks can read package metadata without running package code.

## Scope

- Extract tarball bytes that have already been tied to an M1 artifact identity.
- Reject path traversal and platform-ambiguous paths.
- Reject symlink and hardlink entries that could escape the inspection root.
- Read `package.json` metadata needed by later bin, lifecycle, and dependency
  closure checks.
- Preserve artifact identity throughout extraction.

## First Commit Rule

This trace scaffold is committed before implementation begins.
