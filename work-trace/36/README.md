# Issue 36 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/36

## Request

Download the selected root npm tarball as bytes without executing package code.

## Scope

- Add a testable tarball downloader boundary.
- Fetch bytes from the tarball URL in `ResolvedPackage`.
- Store downloaded bytes in a controlled artifact object for later integrity
  verification.
- Prove the downloader does not call package managers, extract archives, or run
  binaries/lifecycle/dependency scripts.

## First Commit Rule

This trace scaffold is committed before implementation begins.

