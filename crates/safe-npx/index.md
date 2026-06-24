# safe-npx Crate

This crate owns the `safe-npx` CLI.

## Scope

- Parse the package spec and execution policy flags.
- Produce human-readable and JSON scaffold output.
- Prove malformed and unsupported specs stop before network-capable hooks.
- Maintain M1 fixtures for parser, registry, artifact, and no-network behavior.
- Grow into exact package resolution, integrity checks, package evidence extraction, lifecycle-script detection, policy decisions, and fail-closed execution refusal when byte identity cannot be proven.

## Bazel

- Build: `bazel build //crates/safe-npx:safe-npx`
- Root alias: `bazel build //:safe-npx`
