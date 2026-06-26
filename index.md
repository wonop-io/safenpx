# Repository Index

This is the first AI-facing index for `safe-npx`. Keep it short and load child indexes only when the task enters that area.

## Project

`safe-npx` is a Rust execution gate that resolves package evidence before `npx` or `npm exec` runs remote code. The project is early-stage public infrastructure: CLI scaffold, threat model, roadmap, handle reservations, and sponsor-facing launch material.

## Organisation

- `crates/index.md`: first-party Rust crates. Start there for implementation work.
- `docs/`: long-form product and security documentation.
- `policies/index.md`: repository policy checks enforced by local commands and CI.
- `reservations/`: npm and crates.io handle reservation packages.
- `tools/github/index.md`: GitHub planning and Codex workflow helpers.
- `README.md`: public project overview.
- `AGENTS.md`: repository-specific working rules.

## Build

- Canonical: `bazel build //...` and `bazel test //...`
- Policy preflight: `./policies/check.sh`
- Full local check: `just check`
- Rust-native checks: `cargo fmt --check` and `cargo test`
- Root target: `//:safe-npx`
- CLI target: `//crates/safe-npx:safe-npx`

## Key Docs

- `docs/threat-model.md`: security threats and boundaries.
- `docs/demo-flow-spec.md`: demo flow for `safe-npx create-example@latest`.
- `docs/technical-scope.typ`: one-page technical scope source.
- `docs/public-benefit-plan.md`: OSS and ecosystem benefit plan.
- `docs/roadmap.md`: six-month public roadmap and planning operating model.
- `docs/milestones.md`: milestone plan and acceptance criteria.
- `docs/inspect-first-poc.md`: M3 inspect workflow, supported command shapes,
  limits, redaction, and schema compatibility.
- `docs/inspect-json-schema-v0.md`: M3 JSON contract for agents and CI.
- `docs/inspect-latency-budgets.md`: provisional M3 inspect latency budgets and
  fixture/live measurement commands.
- `docs/handle-reservation.md`: npm and crates.io package handle status.
- `CONTRIBUTING.md`: public contributor workflow.
- `SECURITY.md`: security disclosure process.
