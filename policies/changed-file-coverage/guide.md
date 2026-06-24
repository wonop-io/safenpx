# Changed File Coverage Policy

The current coverage gate is repository-wide line coverage at 80% using `cargo llvm-cov`.

Fix by adding tests for changed Rust behavior or intentionally disabling the local expensive gate with `CHANGED_FILE_COVERAGE_ENABLED=0` while iterating.
