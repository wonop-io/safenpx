# Policies Index

Repository policy checks for the public `safe-npx` scaffold. These are enforced by `./policies/check.sh`, `just check`, Bazel targets, the pre-push hook, and GitHub Actions.

## Policy Order

Run policy checks from least expensive to most expensive:

1. `policy-completeness/`: every policy has a check, guide, executable bit, and Bazel wiring.
2. `policy-order/`: `just` entrypoints run cheap policy preflight before formatter, tests, coverage, and Bazel.
3. `ci-entry-consistency/`: CI starts with policy preflight.
4. `just-test-nightly-fmt/`: formatter checks stay wired into `just`.
5. `conventional-commits/`: commit subjects use Conventional Commits.
6. `generated-artifacts/`: build output, caches, logs, coverage, and local artifacts are blocked.
7. `local-secret-paths/`: secret-shaped filenames are blocked.
8. `no-javascript/`: JavaScript source is blocked.
9. `typescript-exceptions/`: TypeScript paths require an explicit exception.
10. `secret-hygiene/`: changed text files are scanned for credential-like material.
11. `sql-repository-boundary/`: database access must use repository boundaries if introduced.
12. `interface-boundary/`: execution and unsafe boundaries require explicit modules.
13. `dead-code/`: dead-code and unused-code suppressions are blocked.
14. `incomplete-production-rust/`: `todo!`, `unimplemented!`, `dbg!`, and `panic!` are blocked in production Rust.
15. `migration-timestamp/`: migration filenames must be timestamped if migrations are added.
16. `migrations-immutable/`: existing migrations are immutable.
17. `dependency-sync/`: Cargo and Bazel dependency metadata stay synchronized.
18. `release-inventory/`: release surfaces stay inventoried.
19. `release-evidence/`: release workflows must preserve evidence before being enabled.
20. `release-provenance/`: release manifests require provenance.
21. `protected-promotions/`: promotion workflows require protected environments.
22. `db-contract-fail-closed/`: database contract tests must fail closed if introduced.
23. `supply-chain-baseline/`: lockfiles and CI safety gates are present.
24. `supply-chain-delta/`: RustSec advisory delta checks run when the tool is installed.
25. `policy-exceptions/`: policy exceptions must be reasoned.
26. `playbook-spec/`: playbooks require a spec before being introduced.
27. `documentation-coverage/`: Rust item documentation coverage must be at least 80%.
28. `index-coverage/`: meaningful source folders need nearby `index.md`.
29. `bazel-ownership/`: meaningful source files belong to Bazel packages.
30. `file-size/`: changed text files stay small enough to review.
31. `replicated-code/`: duplication policy slot is present and ready to tighten as the repo grows.
32. `clean-repo-before-tests/`: optional clean-worktree gate, enabled with `REQUIRE_CLEAN_REPO=1`.
33. `changed-file-coverage/`: coverage gate uses `cargo llvm-cov` when available.

This intentionally mirrors the Wonop policy surface while adapting checks for a small Rust OSS security project.
