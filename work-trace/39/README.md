# Issue 39 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/39

## Request

Expose M1 resolver evidence through the existing human and JSON reports without
claiming the later full evidence schema is complete.

## Scope

- Add stable M1 report fields for command intent, resolved package coordinates,
  registry source, tarball URL, integrity status, digest identity, decision, and
  failure reasons.
- Keep unsupported and malformed specs fail-closed with `downloaded=false`.
- Keep registry and integrity failure states explicit and deterministic.
- Add golden-style tests for a successful report and representative failures.

## First Commit Rule

This trace scaffold is committed before implementation begins.
