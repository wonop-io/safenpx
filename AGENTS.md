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
- `tools/github/`: GitHub Issues, roadmap, and Codex planning helpers.
- `BUILD.bazel` and `MODULE.bazel`: Bazel entrypoints and dependency wiring.

## Planning Workflow

- GitHub Issues and the `safe-npx Roadmap` GitHub Project are the online source of truth for work tracking.
- `docs/roadmap.md` is the durable repo-backed narrative for the six-month plan.
- Before starting any non-trivial work, check whether a GitHub issue already exists for it.
- Do not start feature, bug, security, documentation, or planning work unless it is recorded in a GitHub issue first. Tiny repository chores may be done without a dedicated issue when they are clearly incidental.
- Keep the relevant GitHub issue up to date as work progresses, especially when scope, blockers, acceptance criteria, or status changes.
- Every non-trivial implementation change should start from a GitHub issue.
- Pull requests should link issues with `Closes #123` or `Refs #123`.
- Before starting work, inspect the issue with `just issue-view ISSUE=123` and move it into progress with `just issue-start ISSUE=123`.
- After work is merged or no longer planned, update the issue with `just issue-done ISSUE=123` or close it manually with a clear reason.
- Use `just roadmap-status` to get the current online issue/project snapshot before planning a work session.
- Use conventional commits for every commit, for example `feat:`, `fix:`, `docs:`, `test:`, `refactor:`, or `chore:`.

## Build And Test Workflow

- Bazel is the canonical build and test path. Prefer `bazel build //...` and `bazel test //...`.
- Run `./policies/check.sh` or `just policy-checks` before slower build and test work.
- Keep the policy surface broadly aligned with the Wonop repository policy set; `safe-npx` adapts non-applicable release, database, and playbook policies explicitly rather than silently omitting them.
- Run `just install-hooks` once in a local clone to install the tracked pre-push hook from `.githooks/`.
- Cargo is still useful for Rust-native workflows such as `cargo fmt --check` and local crate iteration.
- When dependencies, crate layout, or package names change, keep Cargo and Bazel wiring in sync.
- Before handing off code changes, run the narrowest useful verification, normally:
  - `cargo fmt --check`
  - `cargo test`
  - `cargo llvm-cov --workspace --all-targets --fail-under-lines 80`
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
