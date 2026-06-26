# Issue 11 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/11

## Request

Extract root package evidence for M3 inspect reports from verified tarball
bytes without running package code.

## Scope

- Compute package size and file count from verified artifact bytes.
- Report package binaries, lifecycle scripts, dependency declarations,
  repository, license, and provenance-like package fields when present.
- Label dependency data as declarations, not verified execution closure.
- Treat missing optional package metadata as absent evidence.
- Preserve the no-execution guarantees proven by #54 and #55.

## First Commit Rule

This trace scaffold is committed before implementation begins.
