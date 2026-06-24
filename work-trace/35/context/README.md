# Issue 35 Context

## Source Inputs

- GitHub issue #35.
- `docs/milestones.md` M1 resolver deliverables.
- Existing M1 data contracts in `crates/safe-npx/src/contracts.rs`.
- Existing parser and fixture manifest work from issues #2, #8, #34, and #38.

## Assumptions

- Public npm registry URL is `https://registry.npmjs.org/` for M1.
- Missing tarball URL or integrity in a selected version is an invalid registry
  response and maps to `registry_error`.
- The first implementation can expose a stub-friendly trait plus a small HTTP
  adapter without wiring live network access into CLI execution.

