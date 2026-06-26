# Issue 56 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/56

## Request

Add registry metadata evidence to M3 inspect reports without treating registry
metadata as verified package contents or execution proof.

## Scope

- Carry selected registry source and package scope category into inspect output.
- Extract publish time, maintainers, publisher, repository, license, dist
  integrity, tarball URL, and provenance-like fields when present.
- Tolerate missing or malformed optional registry metadata.
- Keep registry facts separate from verified tarball/package facts.

## First Commit Rule

This trace scaffold is committed before implementation begins.
