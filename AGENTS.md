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
- `playbooks/`: repository playbooks for repeatable planning and workflow.
- `reservations/`: package-handle reservation packages for npm and crates.io.
- `tools/github/`: GitHub Issues, roadmap, and Codex planning helpers.
- `work-trace/`: checked-in planning artifacts, organized by GitHub issue id.
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
- Use `playbooks/repository/plan-issue.md` when planning an issue before implementation.
- Use conventional commits for every commit, and include the GitHub issue id as the scope whenever it makes sense, for example `feat(37): verify npm integrity`, `fix(8): consume failure fixtures`, or `docs(35): add registry trace`; tiny incidental chores may use an unscoped `chore:`.

## Build And Test Workflow

- Before handing off changes, run `just test`.

## Security Posture

- `safe-npx` is evidence-before-execution infrastructure. Do not add behavior that runs package lifecycle scripts, package binaries, or install hooks before inspection and policy evaluation.
- Tests may use fixtures, but avoid executing third-party package code unless the fixture is explicit, local, and sandboxed.
- Describe results as risk signals, not proof that a package is safe.
- Preserve exact artifact, integrity, dependency graph, lifecycle script, typo-squat, freshness, and maintainer-compromise concerns in user-facing language.

## Style

- Keep the CLI Rust code simple and auditable.
- Prefer explicit data structures over stringly typed policy plumbing.
- Add abstractions only when they make inspection, testing, or policy enforcement clearer.
