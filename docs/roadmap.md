# Public Roadmap

This roadmap is intentionally public and early. The goal is to invite critique before the implementation hardens.

It is also the repo-backed planning narrative for the next six months. GitHub Issues and the `safe-npx Roadmap` GitHub Project are the online source of truth for day-to-day tracking.

## Operating Model

- Public planning happens in GitHub Issues.
- Larger initiatives are parent issues with sub-issues.
- The GitHub Project provides board, table, and roadmap views.
- Milestones group delivery phases.
- PRs link issues with `Closes #123` or `Refs #123`.
- Sponsors and contributors should be able to understand what is planned, what is in progress, and what is blocked without private context.

## Tracks

- CLI: local execution gate, package resolution, terminal UX, JSON output.
- Artifact: npm metadata, tarball identity, integrity, and registry source.
- Policy: local decisions, exit codes, JSON schema, and fail-closed behavior.
- Security: threat model, malicious fixture corpus, supply-chain test cases.
- Docs: contributor docs, package manager notes, agent vendor guidance.
- Community: sponsorship, contributor onboarding, roadmap communication.

## Milestones

### M0: Foundation

Goal: make the project credible, buildable, governed, and easy for Codex and contributors to operate.

- Rust workspace and Bazel build.
- CI, repository policies, documentation coverage, and 80% test coverage.
- GitHub issue templates, roadmap project, labels, milestones, and planning scripts.
- Contributor-facing roadmap and operating docs.

### M1: Package Resolution And Integrity

Goal: resolve a requested package spec to exact npm artifacts without executing package code.

- Parse package specs and package manager intent.
- Resolve npm dist-tags to exact versions.
- Download root tarball without running lifecycle scripts.
- Verify root artifact integrity.
- Produce human and JSON evidence output.

### M2: Execution Closure Spike

Goal: prove whether inspected bytes can be the bytes that later execute.

- Choose direct execution, pinned delegation, or inspect-only alpha.
- Prove no package code runs during inspection.
- Test tag moves, cache poisoning, lifecycle traps, bins, and shims.
- Refuse any command shape that cannot preserve the inspected closure.

### M3: Inspect Evidence And JSON

Goal: make inspect mode useful for humans, agents, and CI.

- Extract root package evidence and label dependency declarations.
- Stabilize JSON schema for agent consumption.
- Add terminal output, redaction rules, and authority-context reporting.
- Keep facts, heuristics, decisions, and reasons separate.

### M4: Policy And Exit Semantics

Goal: turn evidence into predictable local decisions.

- Implement the canonical decision enum and exit code contract.
- Add interactive and non-interactive behavior.
- Make unsupported specs and unverifiable execution fail closed.
- Validate policy rules against fixtures.

### M5: Execute Alpha And Adoption

Goal: run only verified execution closures, or ship inspect-only if proof fails.

- Execute exact-version packages only after M2 proves the path.
- Add approval prompt, experimental approval cache, and compatibility matrix.
- Package an alpha with install, upgrade, and uninstall docs.
- Publish agent, CI, package-author, and contributor docs.
- Demo flow for `safe-npx create-example@latest`.

## Project Fields

Recommended GitHub Project fields:

- Status: Backlog, Ready, In Progress, Review, Done, Blocked.
- Track: CLI, Registry, Policy, Security, Docs, Community.
- Priority: P0, P1, P2, P3.
- Target: M0, M1, M2, M3, M4, M5.
- Effort: S, M, L, XL.
- Risk: Low, Medium, High.
- Start date.
- Target date.

## Initial Issue Shape

Each implementation issue should include:

- Problem.
- Scope.
- Acceptance criteria.
- Security considerations.
- Tests and policy gates.
- Documentation impact.
