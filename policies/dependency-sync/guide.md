# Dependency Sync Policy

Cargo and Bazel dependency metadata must move together.

When `Cargo.toml` changes, run `cargo generate-lockfile` or `cargo test`, then `bazel test //...` so `Cargo.lock` and `MODULE.bazel.lock` exist and are current. When `MODULE.bazel` changes, rerun Bazel and commit the lock update.
