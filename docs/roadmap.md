# Public Roadmap

This roadmap is intentionally public and early. The goal is to invite critique
before the implementation hardens.

## Phase 0: Scope And Safety Language

- Publish the one-page technical scope.
- Publish the threat model.
- Publish the demo-flow spec.
- Publish the public benefit plan.
- Keep claims limited to risk signals, not safety guarantees.

## Phase 1: Rust CLI Prototype

- Parse `safe-npx <package-spec> [-- <args>...]`.
- Resolve package metadata from the NPM registry.
- Download the root tarball without executing package code.
- Verify tarball integrity against registry metadata where available.
- Inspect `package.json`, bins, lifecycle scripts, package size, and file list.
- Print a compact evidence report.
- Support `--json`.
- Do not delegate to `npm exec` until the gate is explicit.

## Phase 2: Dependency Graph And Policy

- Generate a package-lock-only graph with scripts disabled.
- Parse resolved versions, tarball URLs, and integrities.
- Track root and dependency node identities.
- Add local cache keyed by exact artifact integrity.
- Add local policy: allow, ask, deny.
- Add deterministic JSON output for agents.

## Phase 3: Test Corpus And Ecosystem Review

- Add reproducible fixtures for benign packages, lifecycle-script packages,
  typo-squat lookalikes, bundled packages, and intentionally suspicious samples.
- Document expected findings for every fixture.
- Invite package-manager maintainers, security researchers, and agent vendors to
  review the output format.

## Phase 4: Evidence Registry

- Design an optional public evidence registry for exact package artifacts.
- Support verified third-party audit records.
- Support graph-level finding propagation.
- Keep uploads opt-in.
- Provide self-hosting guidance for teams and package infrastructure providers.

