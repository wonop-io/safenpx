# Issue 35 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/35

## Request

Implement the M1 npm metadata client and stable error mapping.

## Scope

- Resolve supported exact-version npm package specs through a registry
  interface that tests can stub.
- Support scoped package metadata URL encoding.
- Return package name, version, registry URL, tarball URL, and integrity when
  registry metadata contains them.
- Map missing packages, missing versions, malformed registry responses, and
  transport failures into stable M1 reasons.

## First Commit Rule

This trace scaffold is committed before implementation begins.

