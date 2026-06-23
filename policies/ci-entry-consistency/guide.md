# CI Entry Consistency Policy

CI and local commands should fail fast on policy problems before running slower format, Cargo, and Bazel work.

Fix by keeping `.github/workflows/ci.yml` and `justfile` wired through `./policies/check.sh`.
