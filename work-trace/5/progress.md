# Issue 5 Progress

## 2026-06-26

- Moved issue #5 to `status:in-progress`.
- Created trace scaffold before implementation.
- Added M3 inspect JSON schema v0 wrapper with `schema_version`, artifact,
  command intent, source context, authority context, facts, heuristics,
  reserved null fields, decision, reasons, next action, null execution, and
  exit code.
- Preserved legacy/additive 0.1 fields during the transition so existing
  callers can migrate without losing old evidence paths.
- Documented schema v0 and compatibility rules in
  `docs/inspect-json-schema-v0.md`.
- Added schema tests for top-level fields, reserved nulls, unsupported input,
  inspection error mapping, and enum vocabulary.
- Added checked-in full JSON golden fixtures for ask, unsupported, and
  inspection error output, and wired them into both Cargo and Bazel test data.
- Addressed red-team review finding that M2 execution-refusal JSON bypassed the
  M3 envelope by adding a dedicated schema wrapper for execution refusals.
- Addressed red-team review finding that transition-era legacy fields were not
  documented by adding them to `docs/inspect-json-schema-v0.md`.
- Verified locally with `cargo fmt --check && just test`.
