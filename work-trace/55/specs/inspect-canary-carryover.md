# Inspect Canary Carryover

## Premise

The M2 harness proves a supplied inspection subject can observe fixture metadata
without executing package traps. M3 must prove the actual `safe-npx inspect`
pipeline preserves that invariant after resolve, verify, extract, and render.

## Test Shape

The tests should:

- build local verified tarball fixtures whose `package.json` contains the same
  trap surfaces as the M2 canary manifest
- resolve those fixtures through stub registry and tarball transports
- run the explicit inspect action through `build_report_with_resolver` and
  `render_report`
- assert every sentinel from the fixture remains absent after both human and
  JSON rendering
- include an actionable assertion message naming the fixture and trap kind

## Boundaries

- The canary tarballs may contain trap-looking metadata and files, but no test
  may invoke Node, shell scripts, npm, npx, package binaries, or generated shims.
- The network-attempt trap is represented as package metadata/files only; this
  issue does not add outbound network probing to the inspect pipeline.
- Stable decision/reason modeling for lifecycle and dependency blockers remains
  owned by #57/#58.
