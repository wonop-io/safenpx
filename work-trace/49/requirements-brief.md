# Requirements Brief

## Acceptance Criteria

- Prototype executes only local fixture packages created inside the test
  harness.
- Selected bin bytes are verified and recorded before execution.
- Forwarded args are preserved byte-for-byte in fixture assertions.
- Packages with dependencies, lifecycle scripts, ambiguous bins, missing bins,
  or shim ambiguity return `execution_refused`.
- Prototype does not invoke `npm`, `npx`, package-manager install, or raw shell
  fallback.

## Verification

- `just test`

