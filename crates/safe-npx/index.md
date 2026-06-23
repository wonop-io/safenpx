# safe-npx Crate

This crate owns the `safe-npx` CLI.

## Scope

- Parse the package spec and execution policy flags.
- Produce human-readable and JSON scaffold output.
- Grow into exact package resolution, integrity checks, dependency graph inspection, lifecycle-script detection, policy decisions, and eventual `npm exec` delegation.

## Bazel

- Build: `bazel build //crates/safe-npx:safe-npx`
- Root alias: `bazel build //:safe-npx`
