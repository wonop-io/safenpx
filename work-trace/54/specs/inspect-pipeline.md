# Inspect Pipeline

## First Slice

The first M3 inspect pipeline should introduce an explicit inspect command path
without expanding execution. It can reuse the existing M1 resolver and report
model, but the CLI surface must make inspection the named action.

## Required Behavior

- `safe-npx inspect <exact-spec>` is accepted.
- Legacy direct `safe-npx <spec>` behavior remains inspect-like and
  non-executing for compatibility with earlier scaffold tests.
- Unsupported and malformed specs stop before network-capable hooks.
- Successful exact-version inspection resolves, downloads, verifies, extracts
  static metadata, renders output, and stops.
- JSON and human output are rendered from the same inspect result.
- No package binaries, lifecycle scripts, dependency scripts, npm install,
  npm exec, raw `npx`, or shell fallback may run.

## Deferred Beyond This Issue

- Full M3 evidence model fields are #57.
- Human report richness is #58.
- JSON schema v0 and golden compatibility are #5 and #59.
- No-package-code canaries against the complete inspect path are #55.
