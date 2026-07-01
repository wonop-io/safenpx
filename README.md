# safenpx

`safe-npx` is a Rust-based execution gate for `npx` / `npm exec`.

The default `npx` prompt is too thin. It can be about to install and execute
remote package code, but it gives almost no evidence: not who published the
latest release, not how old it is, not whether it has lifecycle scripts, not
whether it is bundled or obfuscated, not whether the name looks suspicious, and
not whether the exact artifact has been audited.

`safe-npx` keeps the workflow and fixes the decision point.

```bash
safe-npx inspect create-example@1.2.3
```

In the M3 proof of concept, inspect mode resolves the exact package artifact,
verifies integrity, inspects registry metadata and package contents, then stops.
It does not run package binaries, lifecycle scripts, dependency scripts, or raw
`npx`.

For agents:

```bash
safe-npx --json inspect create-example@1.2.3
```

The JSON output lets a coding agent stop, explain the risk, and ask the user
before running remote package code.

## Status

Early public scaffold with an inspect-first proof of concept. This repository
contains the product scope, threat model, public benefit plan, demo-flow spec,
Rust CLI, inspect evidence model, human report, and JSON schema V0. The first
working prototype is intentionally narrow.

## v0.1 Goal

Make the current `npx` permission prompt materially more useful before any
package code runs.

The first release should:

- Parse supported exact-version package specs.
- Download and verify the root package tarball.
- Inspect root package metadata and contents without running lifecycle scripts.
- Treat dependency declarations as evidence until dependency closure is fully
  resolved and verified.
- Show publisher, release age, package size, bins, lifecycle scripts, and
  suspicious metadata.
- Flag typo-squatting, recently changed packages, and bundled or obfuscated
  code.
- Emit readable output for humans and JSON policy output for agents.
- Refuse unsupported or unverifiable command shapes instead of falling back to
  raw `npx`.

## Repository Map

- `AGENTS.md` and `index.md`: AI-facing repository workflow and navigation.
- `CONTRIBUTING.md`: contributor workflow, issue-backed work, and verification.
- `SECURITY.md`: vulnerability reporting and security expectations.
- `MODULE.bazel` and `BUILD.bazel`: Bazel module and root build targets.
- `crates/safe-npx`: Rust CLI scaffold.
- `docs/technical-scope.typ` and `docs/technical-scope.pdf`: one-page technical scope.
- `docs/threat-model.md`: threat model and boundaries.
- `docs/demo-flow-spec.md`: demo script specification, not yet implemented.
- `docs/inspect-first-poc.md`: M3 inspect workflow, supported command shapes,
  limits, redaction, and schema compatibility.
- `docs/inspect-json-schema-v0.md`: agent and CI JSON output contract.
- `docs/m4-policy-v0.md`: provisional M4 policy, thresholds, next actions,
  exit codes, and representative outcomes.
- `docs/inspect-latency-budgets.md`: provisional M3 inspect latency budgets and
  measurement commands.
- `docs/public-benefit-plan.md`: OSS, test corpus, and ecosystem benefit plan.
- `docs/roadmap.md`: public roadmap.
- `docs/one-year-vision.md`: one-year product and architecture vision.
- `docs/milestones.md`: implementation milestones derived from the one-year vision.
- `docs/handle-reservation.md`: npm and crates.io package handle status.

## Planning

Work is tracked in GitHub Issues and the public
[safe-npx Roadmap](https://github.com/orgs/wonop-io/projects/6).

Non-trivial work should start from an issue. The next ready tasks can be found
in the [M0 milestone](https://github.com/wonop-io/safenpx/milestone/1) and
later milestone queues.

## Safety Language

`safe-npx` does not prove that a package is safe. It provides evidence-backed
risk signals before execution.

M3 is inspect-first. It can collect evidence for supported command shapes, but
general package execution is a later milestone. Unsupported or unverifiable
commands must fail closed rather than falling back to raw `npx`.

Prefer:

- evidence before execution
- exact artifact integrity
- risk signals, not guarantees
- human-readable and agent-readable decisions

Avoid:

- "this makes NPX safe"
- "this proves packages are safe"
- "AI-powered NPM malware scanner"

## License

This project is licensed under the Apache License, Version 2.0.
See `LICENSE.md`.
