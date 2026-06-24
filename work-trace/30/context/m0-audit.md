# M0 Audit Evidence

## Current Open M0 Issues

Audited on 2026-06-24:

- #1 Keep public GitHub roadmap workflow current
- #18 Establish Rust workspace and Bazel build scaffold
- #19 Install repository policies, pre-push hook, and CI gates
- #20 Publish baseline roadmap, threat model, scope, and vision docs
- #21 Configure GitHub milestones, labels, templates, and seed issues
- #22 Document issue-backed contributor and agent workflow
- #23 Verify README and index navigation for new contributors
- #24 Track npm and crates.io handle reservations
- #25 Align public README scope with exact-version-first plan
- #26 Add contributor and security policy docs
- #27 Tighten issue and PR templates for proof-obligation work
- #28 Populate the public roadmap project with seeded issues
- #29 Verify package handle reservations and ownership
- #30 Audit and close completed foundation tickets

## Evidence Collected

- `just test` passed before the issue #30 trace commit.
- Public roadmap project #6 contains issues #1 through #30.
- npm package views succeeded for `safe-npx`, `@wonop/safe-npx`, and
  `@wonop/safenpx`.
- crates.io package info succeeded for `safe-npx` and `safenpx`.
- README, root index, crate index, and technical scope were aligned with the
  exact-version-first and fail-closed milestone strategy.
- Contributor and security docs were added.
- Issue and PR templates were expanded with proof-obligation, verification,
  exit-code, fail-closed, and documentation-impact prompts.

