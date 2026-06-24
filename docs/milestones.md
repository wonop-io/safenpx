# Milestones

This document turns the one-year vision into implementation milestones.

The first product slice is deliberately narrower than the full package-security
conversation in Theo's video. `safe-npx` should first become the checkpoint
before remote package code runs: resolve the exact package, inspect it without
running it, decide with evidence, and execute only when the same inspected bytes
can be proven to run.

Broader ideas such as hosted audit records, release reputation, badges, and
registry-scale governance matter later. They should not distract from the first
proof: blind `npx` execution gets replaced with an evidence gate.

## Reader Model

For every supported command, `safe-npx` follows this model:

1. Resolve the requested package to exact coordinates.
2. Download the exact tarball.
3. Verify integrity and record artifact identity.
4. Inspect metadata and package contents without running package code.
5. Decide: allow, ask, deny, or stop because more proof is required.
6. Execute only if the inspected bytes are the bytes that will run.

Execution closure means every local artifact that can execute as part of the
command: the root tarball, selected binary, generated shim, installed
dependencies, lifecycle scripts, package-manager helpers, and any resolved files
needed before the command starts. Dependency declarations are not an execution
closure; they are only evidence until the resolver proves and verifies them.

## Milestone Principles

- Prove the execution boundary before expanding the product surface.
- Keep the first useful path narrow: common npm package specs, root evidence,
  inspect mode, JSON output, exact-version execution, and clear refusal.
- Treat v0 policy thresholds as provisional constants until fixtures and
  dogfooding prove they change decisions.
- Start fixtures with the resolver instead of treating QA as a late phase.
- Keep hosted audit records, broad private registry support, and alternate
  package managers out of the critical path.
- Every milestone should leave behind tests, fixture evidence, and docs.

## Ownership Lanes

- CLI/runtime: argument parsing, modes, prompts, exit codes, and process launch.
- Registry/artifact: npm metadata, tarballs, integrity, cache provenance, and
  registry precedence.
- Execution: closure spike, direct execution/delegation proof, and no-run
  invariants.
- Policy/evidence: facts, heuristics, decisions, JSON schema, and redaction.
- Fixtures/QA: malicious fixtures, lifecycle traps, tag moves, and regressions.
- Docs/adoption: agent guides, CI guides, demo, README workflow, and author
  guidance.

GitHub issues should be cut by proof obligation, not broad theme. Each issue
should include owner lane, dependency, CLI surface, fixture, acceptance criteria,
and exit-code behavior.

## M0: Repository And Planning Baseline

Goal: make the project buildable, governed, and easy to plan in public.

Status: mostly complete.

Deliverables:

- Rust workspace and Bazel build.
- Repository policies, pre-push hook, CI, documentation coverage, and 80% test
  coverage gate.
- Public roadmap, threat model, demo spec, public benefit plan, and one-year
  vision.
- GitHub issue templates, labels, project fields, and planning helpers.

Definition of done:

- `./policies/check.sh` passes locally and in CI.
- `bazel test //...` passes in CI.
- New contributors can find the vision, roadmap, milestones, and issue workflow
  from the README.

## M1: Resolver, Artifact Identity, And Fixture Seed

Goal: turn a supported `safe-npx` command into exact package coordinates and a
verified root artifact without running package code.

Supported v0 inputs:

- `name@version`
- `@scope/name@version`
- Forwarded package arguments after `--`

Supported after the tag-race proof:

- `name@latest`
- `@scope/name@latest`

Explicitly unsupported in v0:

- Unversioned names.
- Version ranges other than `latest`.
- Git URLs, local paths, tarball URLs, aliases, and multiple package specs.
- `npm exec --package`, `-c`, and package-manager-specific variants.

Deliverables:

- Package-spec parser with a supported/unsupported matrix.
- npm registry client for public package metadata.
- Exact-version resolution.
- Tarball download without lifecycle execution.
- Integrity verification and digest identity.
- Fixture manifest seed for parser, registry errors, integrity mismatch,
  malformed specs, and forwarded args.
- Unsupported-spec refusal messages that say what was rejected and whether
  anything was downloaded.

Acceptance criteria:

- Supported specs resolve to exact package name, version, registry, tarball URL,
  and digest.
- Integrity mismatch returns `deny` and exits without execution.
- Unsupported specs fail closed and never silently call raw `npx`.
- Malformed specs cause no network calls.
- Resolver fixtures cover scoped packages, missing packages, missing versions,
  registry errors, malformed specs, integrity mismatch, and forwarded args.

## M2: Execution Closure Spike

Goal: prove whether `safe-npx` can inspect the same bytes that later execute.

This is the highest-risk milestone. If a command shape cannot satisfy this
invariant, the alpha must refuse that shape instead of delegating to raw `npx`.

V0 stance:

- Execute exact-version package specs first.
- Run from a verified local closure controlled by `safe-npx`.
- Deny dependency lifecycle scripts by default until they are part of the
  verified closure.
- Treat dependency declarations as evidence only unless dependencies are
  resolved, downloaded, integrity-checked, and included in the closure.
- Add `latest` only after tag-move and refetch races fail closed.

Deliverables:

- Execution-closure design note comparing direct execution, pinned local
  tarball execution, and package-manager delegation.
- No-package-code-ran canary harness using sentinel files, environment markers,
  and blocked network attempts.
- Race matrix for resolution time versus execution time.
- Bin selection rules and refusal behavior for ambiguous or missing bins.
- Registry and `.npmrc` precedence tests for public npm and scoped registry
  smoke cases.
- Closure fixtures for tag moves, cache poisoning, dependency lifecycle escape,
  generated shims, and selected binaries.

Acceptance criteria:

- Inspection cannot run package binaries, lifecycle scripts, or dependency
  scripts.
- The selected execution mechanism cannot inspect one version and execute
  another.
- Registry metadata changes between inspection and execution fail closed.
- Cache entries, generated shims, selected bins, dependencies, and lifecycle
  scripts cannot escape the verified closure.
- If full dependency closure cannot be proven, execution refuses with a clear
  unsupported-closure result.

## M3: Inspect Evidence And JSON V0

Goal: produce useful human and machine evidence before any package code can run.

Deliverables:

- Inspect mode that resolves, downloads, verifies, extracts evidence, and stops.
- Static extraction from registry metadata, tarball metadata, and `package.json`.
- Human report separating facts, provisional heuristics, decision, and authority
  context.
- Versioned JSON schema with deterministic ordering and golden fixture outputs.
- JSON fields for `schema_version`, `artifact`, `command_intent`,
  `source_context`, `authority_context`, `facts`, `heuristics`,
  `external_evidence`, `attestations`, `release_diff`, `decision`, `reasons`,
  `required_next_action`, and `exit_code`.
- Privacy and redaction rules for reports and JSON.

Evidence v0:

- Requested command, selected bin, and forwarded args.
- Source context: manual terminal, README/docs snippet, agent skill, CI, or
  unknown.
- Resolved package identity and integrity verification.
- Registry source and publish time.
- Publisher, maintainers, repository, license, and provenance fields when
  available.
- Package size, file count, binaries, and lifecycle scripts.
- Dependency declarations, clearly labeled as declarations unless included in
  the verified execution closure.
- Similar-name and unusual-shape heuristics as report-only signals.

Acceptance criteria:

- Inspect mode never runs package binaries, lifecycle scripts, or dependency
  scripts.
- JSON output is deterministic for fixture packages and has compatibility
  tests.
- Reports never print secret values, private registry tokens, or full sensitive
  environment details.
- Authority context includes registry source, package scope, command intent, and
  cwd trust class, but uses categories and redacted names rather than raw secret
  values, full environment dumps, or home paths.
- The report plainly states what `safe-npx` catches and does not catch.
- Latency is measured for cold cache and warm cache runs.

## M4: Provisional Policy V0 And Exit Semantics

Goal: turn evidence into predictable allow, ask, deny, and stop outcomes.

Policy v0 is intentionally provisional. Thresholds should be constants that are
easy to change after fixture and dogfood feedback.

Initial thresholds:

- Recent publish warning: package version published within 24 hours.
- Large package warning: tarball larger than 5 MB.
- Large file-count warning: more than 500 files.
- Lifecycle script: ask in interactive mode, stop in non-interactive mode.
- Integrity mismatch, unsupported closure, or resolver ambiguity: deny.

Deliverables:

- Interactive and non-interactive modes.
- Exit codes for successful inspection, ask-required, deny, unsupported input,
  inspection error, execution refused, and delegated execution failure.
- Example outcomes for `allow`, `ask`, `deny`, `unsupported`,
  `inspection_error`, `execution_refused`, and non-interactive stop.
- Provisional policy fixtures covering every threshold and fail-closed path.
- Agent-readable JSON decision semantics for both interactive and
  non-interactive mode.

Acceptance criteria:

- Non-interactive mode stops when policy requires a question.
- Similar-name and unusual-shape heuristics remain report-only until validated.
- Every policy rule has pass/fail fixtures.
- Fixture output explains whether the user can retry with a narrower command,
  inspect-only mode, or an explicit override.

## M5: Execute Mode Alpha

Goal: safely run the intended command only after policy allows the verified
execution closure.

Deliverables:

- Execute mode for exact-version specs that pass the closure proof.
- Later support for `latest` after tag-race fixtures pass.
- Approval prompt for interactive mode.
- Explicit inspect-only behavior for `--json` unless an execute path is chosen.
- Local approval cache keyed by artifact digest, command intent, policy version,
  registry source, authority context, and expiry.
- Cache invalidation on digest, policy, registry source, authority context, or
  expiry changes.
- Compatibility matrix showing common `npx` commands that alpha supports,
  refuses, or intentionally does not emulate.
- Alpha package distribution for dogfooding.

Acceptance criteria:

- Saying no runs nothing from the package.
- Saying yes runs only from the verified execution closure.
- Approval in one authority context does not silently apply to a materially
  broader context.
- `latest` cannot inspect one version and execute another.
- Dogfood runs record latency, unsupported command shapes, policy decisions,
  bypass pressure, and workaround behavior.

## M6: Expanded Fixture Corpus And Decision Validation

Goal: prove that evidence changes behavior without creating unacceptable noise.

Deliverables:

- Fixture manifest format with package, digest, execution closure, expected
  evidence, expected policy result, and whether execution is allowed.
- Benign CLI fixtures.
- Lifecycle-script fixtures.
- Fresh-release fixtures.
- Maintainer-compromise simulations.
- Typo-like name fixtures.
- Obfuscated root package fixtures.
- Dependency-declaration and dependency-confusion fixtures.
- Cache/tag-move fixtures.
- Real incident replay fixtures where legally and ethically usable.
- Lightweight human and agent decision studies.

Acceptance criteria:

- Fixture runs prove package code does not execute during inspection.
- Five to ten developers compare raw `npx` prompts with `safe-npx` reports.
- Agent tests show JSON causes stop, explain, approval request, or fail-closed
  behavior.
- Maintainer feedback identifies whether package authors would change package
  shape for cleaner reports.
- Time-to-decision, false-positive, false-negative, and bypass-pressure notes
  are published.

## M7: Agent, CI, Author, And Early Adoption

Goal: make the tool usable in the places blind package execution is most
dangerous.

Deliverables:

- Agent integration guide for `safe-npx --json` inspect mode.
- CI examples for fail-closed package execution.
- Copy-paste replacement snippets for READMEs, agent skills, docs pages, and CI
  scripts.
- Author guidance for safe-npx-friendly CLI packages: clear bins, stable
  releases, minimal lifecycle scripts, provenance fields, and readable metadata.
- Terminal-first demo comparing raw `npx` with `safe-npx`.
- Early adopter feedback from developers, agent-tool builders, package-security
  maintainers, and small CLI authors.

Acceptance criteria:

- Agents can render an approval request without scraping terminal text.
- CI can fail closed on deny, unsupported, inspection error, or ask-required.
- Early adopters can install and run the alpha on real commands.
- Feedback identifies whether hosted audit records, private registry support,
  or release diffing are justified next.

## Deferred Until Core Proof Works

These are real opportunities, but not core milestone blockers:

- Hosted audit records and public artifact pages.
- Third-party audit attestations.
- Author-facing audit badges.
- Release diff mode beyond placeholder fields.
- Broad private registry productization.
- `pnpm dlx`, `yarn dlx`, `bun x`, and other ecosystems.
- Runtime sandbox profiles.
- Package-name dispute or registry governance workflows.

## Milestone Dependency Order

1. M1 blocks artifact evidence.
2. M2 blocks safe execution and decides which command shapes can ever run.
3. M3 blocks stable agent and CI integration.
4. M4 blocks predictable prompting, fail-closed automation, and approval cache.
5. M5 blocks external dogfooding.
6. M6 validates whether the policy and evidence are worth keeping.
7. M7 should start only after inspect mode, JSON, and fail-closed behavior are
   stable enough for external feedback.
