# Issue 37 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/37

## Request

Verify npm integrity metadata for downloaded root tarball bytes and compute a
stable artifact identity.

## Scope

- Verify npm `dist.integrity` for supported algorithms.
- Return `deny` with `integrity_mismatch` when bytes do not match metadata.
- Compute a stable digest identity for the exact downloaded bytes.
- Cover valid, mismatch, missing, and unsupported integrity metadata.

## First Commit Rule

This trace scaffold is committed before implementation begins.

