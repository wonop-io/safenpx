# Requirements Brief

## Acceptance Criteria

- Root lifecycle script presence returns `execution_refused` with reason `lifecycle_script_present` for execute candidates.
- Any dependency declaration that would require install/resolve work returns `execution_refused` with reason `unsupported_closure` until dependency closure proof exists.
- Optional and peer dependency cases are explicitly classified.
- Bundled dependency metadata is not assumed safe without fixture evidence.
- Tests cover no-deps/no-scripts, lifecycle scripts, dependencies, optional dependencies, peer dependencies, and bundled dependencies.

## Verification

- `just test`
