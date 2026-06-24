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
- Registry: audit repository, package-version evidence, dependency snowballing.
- Policy: risk scoring, local policy files, allow/deny/ask decisions.
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

### M2: Dependency Graph And Evidence

Goal: show what would enter the execution environment before package code runs.

- Resolve dependency graph for the selected root artifact.
- Track integrity where available for dependency nodes.
- Identify bins, lifecycle scripts, bundled code, package age, and metadata anomalies.
- Add fixture corpus for malicious packages, compromised maintainers, typo-squats, lifecycle scripts, and agent-driven blind execution.

### M3: Policy Engine And Agent UX

Goal: make `safe-npx` useful for humans, agents, and CI.

- Implement allow/ask/deny policy decisions.
- Support local policy configuration.
- Stabilize JSON schema for agent consumption.
- Add explainable risk signals and terminal output.
- Support dry-run and fail-closed modes.

### M4: Audit Registry Alpha

Goal: create a registry model for package-version audit trails and dependency snowballing.

- Define audit record schema for package/version artifacts.
- Store package evidence, integrity, timestamps, and risk findings.
- Link findings across dependencies.
- Design registry sync/update workflow.
- Publish initial reproducible corpus and examples.

### M5: Public Beta And Ecosystem Adoption

Goal: make the project ready for external contributors, sponsors, and early agent/package-manager integrations.

- Public beta release.
- Contributor guide and governance.
- Package manager and agent vendor docs.
- Sponsor-facing roadmap and recognition model.
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
