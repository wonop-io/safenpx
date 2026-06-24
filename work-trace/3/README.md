# Issue 3 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/3

## Request

Resolve a supported exact package spec into a verified root npm artifact identity
without running package code.

## Scope

- Compose the M1 parser, npm metadata client, tarball downloader, and integrity
  verifier.
- Return resolved package metadata and stable artifact identity on success.
- Return stable M1 failure reasons for missing package, missing version,
  registry/download failures, and integrity mismatch.
- Preserve the no-execution boundary proven in earlier M1 tickets.

## First Commit Rule

This trace scaffold is committed before implementation begins.

