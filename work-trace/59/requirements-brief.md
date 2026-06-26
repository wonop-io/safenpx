# Requirements Brief

## Problem

M3 inspect JSON is an agent-facing contract. The contract needs checked-in
golden fixtures that prove stable output for representative safe-npx decisions
and fail loudly when compatibility-critical fields drift.

## Acceptance Mapping

- Golden fixtures are checked in and consumed by tests.
- Re-running tests produces byte-stable JSON for the same fixture inputs.
- Fixtures cover successful inspection, unsupported input, integrity failure,
  lifecycle/dependency blockers, redacted authority context, and missing
  optional metadata.
- Compatibility tests fail when enum semantics change without a documented
  schema bump or migration note.
- JSON rendering does not depend on host-specific ordering, temp paths, locale,
  timezone, or current clock.

## Constraints

- Keep fixture data deterministic and local.
- Avoid host paths, live network, current time, and ambient environment.
- Keep Cargo and Bazel wiring in sync.
- Preserve the M3 inspect schema from issue #5.

