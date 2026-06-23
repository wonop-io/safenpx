# AGENTS

This repository is the public scaffold for `safe-npx`, a Rust execution gate for `npx` and `npm exec`.

## Context Loading

- Start with `index.md`, then load the nearest child `index.md` for the area being changed.
- Keep markdown outside `docs/` concise and navigational.
- Put durable human documentation in `docs/`; put AI-facing orientation in local `index.md` files.

## Layout

- `crates/`: first-party Rust crates. The CLI lives in `crates/safe-npx`.
- `docs/`: product scope, roadmap, threat model, demo spec, and public benefit plan.
- `policies/`: repository policy checks wired through Bazel, `just`, and CI.
- `reservations/`: package-handle reservation packages for npm and crates.io.
- `BUILD.bazel` and `MODULE.bazel`: Bazel entrypoints and dependency wiring.

## Build And Test Workflow

- Bazel is the canonical build and test path. Prefer `bazel build //...` and `bazel test //...`.
- Run `./policies/check.sh` or `just policy-checks` before slower build and test work.
- Cargo is still useful for Rust-native workflows such as `cargo fmt --check` and local crate iteration.
- When dependencies, crate layout, or package names change, keep Cargo and Bazel wiring in sync.
- Before handing off code changes, run the narrowest useful verification, normally:
  - `cargo fmt --check`
  - `cargo test`
  - `bazel test //...`
  - `git diff --check`

## Security Posture

- `safe-npx` is evidence-before-execution infrastructure. Do not add behavior that runs package lifecycle scripts, package binaries, or install hooks before inspection and policy evaluation.
- Tests may use fixtures, but avoid executing third-party package code unless the fixture is explicit, local, and sandboxed.
- Describe results as risk signals, not proof that a package is safe.
- Preserve exact artifact, integrity, dependency graph, lifecycle script, typo-squat, freshness, and maintainer-compromise concerns in user-facing language.

## Style

- Keep the CLI Rust code simple and auditable.
- Prefer explicit data structures over stringly typed policy plumbing.
- Add abstractions only when they make inspection, testing, or policy enforcement clearer.
