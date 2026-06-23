# safenpx

`safe-npx` is a Rust-based execution gate for `npx` / `npm exec`.

The default `npx` prompt is too thin. It can be about to install and execute
remote package code, but it gives almost no evidence: not who published the
latest release, not how old it is, not whether it has lifecycle scripts, not
whether it is bundled or obfuscated, not whether the name looks suspicious, and
not whether the exact artifact has been audited.

`safe-npx` keeps the workflow and fixes the decision point.

```bash
safe-npx create-example@latest
```

Before execution, the tool should resolve the exact package artifact, verify
integrity, inspect metadata and package contents, summarize the dependency
graph, apply local policy, and ask before delegating to `npm exec`.

For agents:

```bash
safe-npx --json create-example@latest
```

The JSON output lets a coding agent stop, explain the risk, and ask the user
before running remote package code.

## Status

Early public scaffold. This repository contains the product scope, threat model,
public benefit plan, demo-flow spec, and a Rust CLI skeleton. The first working
prototype is intentionally narrow.

## v0.1 Goal

Make the current `npx` permission prompt materially more useful before any
package code runs.

The first release should:

- Resolve the requested package to an exact version.
- Download and verify the root package tarball.
- Generate a dependency graph without running lifecycle scripts.
- Track integrity for dependency nodes where available.
- Show publisher, release age, package size, bins, lifecycle scripts, and
  suspicious metadata.
- Flag typo-squatting, recently changed packages, and bundled or obfuscated
  code.
- Emit readable output for humans and JSON policy output for agents.
- Delegate to `npm exec` only after policy or the user allows it.

## Repository Map

- `docs/technical-scope.typ` and `docs/technical-scope.pdf`: one-page technical scope.
- `docs/threat-model.md`: threat model and boundaries.
- `docs/demo-flow-spec.md`: demo script specification, not yet implemented.
- `docs/public-benefit-plan.md`: OSS, test corpus, and ecosystem benefit plan.
- `docs/roadmap.md`: public roadmap.
- `src/main.rs`: Rust CLI scaffold.

## Safety Language

`safe-npx` does not prove that a package is safe. It provides evidence-backed
risk signals before execution.

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

This project is intended to be released under a permissive open-source license.
See `LICENSE.md`.

