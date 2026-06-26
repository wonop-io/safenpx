# Human Inspect Report

## Sections

The human report should make these boundaries visible:

- Requested command and resolved artifact facts.
- Registry and package metadata facts.
- Static extraction facts.
- Provisional heuristics.
- Authority context.
- Decision, reasons, required next action, and safety boundary.

## Evidence

Include evidence when available:

- requested command and forwarded args
- selected package/version and artifact identity
- registry source and optional registry metadata
- package size and file count
- bin declarations
- lifecycle scripts
- dependency declarations labeled as unverified declarations
- maintainers, publisher, repository, license, and provenance

## Safety Boundary

The report must state that safe-npx catches unsupported command shapes,
integrity mismatches, lifecycle/dependency surfaces, and authority-context risk
signals, but does not prove package safety or verify dependency closure in M3.

## Verification

- Golden human snapshots cover normal evidence, blockers, unsupported input,
  missing optional metadata, and redacted authority context.
- `just test`

