# Issue 5 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/5

## Request

Define inspect JSON schema v0 so agents and CI can decide from structured
output instead of scraping terminal text.

## Scope

- Add `schema_version` and the full M3 top-level JSON surface.
- Keep inspect-mode `execution` null.
- Reserve nullable `external_evidence`, `attestations`, and `release_diff`.
- Keep enum spelling aligned with the M3 milestone vocabulary.
- Document compatibility rules for additive fields, enum additions, and enum
  semantic changes.
- Add tests that make the schema shape and compatibility rules executable.

## First Commit Rule

This trace scaffold is committed before implementation begins.
