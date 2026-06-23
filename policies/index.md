# Policies Index

Repository policy checks for the public `safe-npx` scaffold. These are enforced by `./policies/check.sh`, `just check`, Bazel targets, and GitHub Actions.

## Policy Order

Run cheap structural checks before expensive Rust and Bazel work:

1. `policy-completeness/`: every policy has a check, guide, executable bit, and Bazel wiring.
2. `ci-entry-consistency/`: CI and `justfile` run policy preflight before build/test work.
3. `generated-artifacts/`: blocks build output, caches, logs, and local artifacts.
4. `local-secret-paths/`: blocks secret-shaped filenames.
5. `secret-hygiene/`: scans changed text files for credential-like material.
6. `dependency-sync/`: keeps Cargo and Bazel dependency metadata in sync.
7. `documentation-coverage/`: requires Rust code documentation coverage of at least 80%.
8. `index-coverage/`: keeps source areas covered by nearby `index.md`.
9. `bazel-ownership/`: keeps meaningful source files inside Bazel packages.
10. `incomplete-production-rust/`: blocks new `todo!`, `unimplemented!`, `dbg!`, and `panic!` macros in production Rust.

Use `./policies/check.sh --base <ref>` to compare against a specific base. Without `--base`, checks use `COMPARE_BRANCH`, then `origin/main`, then `main`, then `HEAD`.
