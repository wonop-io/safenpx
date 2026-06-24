# Issue 38 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/38

## Request

Add a malformed-spec no-network test harness for M1.

## Scope

- Prove malformed specs make zero registry calls.
- Prove unsupported specs make zero tarball download calls.
- Keep the harness reusable for parser, resolver, and fixture-manifest tests.
- Record the no-network fixture expectation.

## First Commit Rule

This trace scaffold is committed before implementation begins.
