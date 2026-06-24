# Milestones

This document turns the one-year vision into implementation milestones.

The plan is intentionally ordered around the hardest proof first: `safe-npx`
must inspect the same execution closure that can later run. If that invariant
cannot be proven for a command shape, the tool must refuse instead of falling
back to raw `npx`.

## Milestone Principles

- Prove the execution boundary before expanding the product surface.
- Keep the first useful path narrow: common npm package specs, root evidence,
  inspect mode, JSON output, and clear refusal.
- Treat heuristics as experiments until fixtures prove they change decisions.
- Keep hosted audit records, broad private registry support, and alternate
  package managers out of the critical path.
- Every milestone should leave behind tests, fixture evidence, and docs.

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
- New contributors can find the vision, roadmap, and issue workflow from the
  README.

## M1: Resolver And Artifact Identity

Goal: turn a supported `safe-npx` command into exact package coordinates and a
verified root artifact without running package code.

Supported v0 inputs:

- `name`
- `name@version`
- `name@latest`
- `@scope/name`
- `@scope/name@version`
- `@scope/name@latest`
- Forwarded package arguments after `--`

Explicitly unsupported in v0:

- Version ranges other than `latest`.
- Git URLs, local paths, tarball URLs, aliases, and multiple package specs.
- `npm exec --package`, `-c`, and package-manager-specific variants.

Deliverables:

- Package-spec parser with a supported/unsupported matrix.
- npm registry client for public package metadata.
- Dist-tag and exact-version resolution.
- Tarball download without lifecycle execution.
- Integrity verification and digest identity.
- Unsupported-spec refusal messages that say what was rejected and whether
  anything was downloaded.

Acceptance criteria:

- Supported specs resolve to exact package name, version, registry, tarball URL,
  and digest.
- Integrity mismatch returns `deny` and exits without execution.
- Unsupported specs fail closed and never silently call raw `npx`.
- Resolver fixtures cover scoped packages, missing packages, missing versions,
  registry errors, malformed specs, and forwarded args.

## M2: Inspect Mode And Evidence Report

Goal: produce useful human and JSON evidence before any package code can run.

Deliverables:

- Inspect mode that resolves, downloads, verifies, extracts evidence, and stops.
- Static extraction from registry metadata, tarball metadata, and `package.json`.
- Human report separating facts, heuristics, decision, and authority context.
- Minimal JSON schema with `artifact`, `command_intent`, `authority_context`,
  `facts`, `heuristics`, `decision`, `reasons`, `required_next_action`, and
  `exit_code`.
- Privacy rules for reports and JSON.

Evidence v0:

- Requested command and forwarded args.
- Resolved package identity and integrity verification.
- Registry source and publish time.
- Publisher, maintainers, repository, license, and provenance fields when
  available.
- Package size, file count, binaries, and lifecycle scripts.
- Dependency declarations, clearly labeled as declarations rather than a fully
  verified dependency execution closure.
- Similar-name and unusual-shape heuristics as report-only signals.

Acceptance criteria:

- Inspect mode never runs package binaries, lifecycle scripts, or dependency
  scripts.
- JSON output is deterministic for fixture packages.
- Reports never print secret values, private registry tokens, or full sensitive
  environment details.
- The report plainly states what `safe-npx` catches and does not catch.

## M3: Policy V0 And Execution Closure Spike

Goal: decide whether execution is allowed, and prove how inspected bytes become
executed bytes.

Deliverables:

- Hardcoded policy v0 using the one-year vision threshold table.
- Interactive and non-interactive modes.
- Exit codes for successful execution, ask-required, deny, unsupported input,
  inspection error, and delegated execution failure.
- Execution closure design spike.
- Pass/fail tests for the chosen execution mechanism.

Execution closure options to evaluate:

- Direct execution from a verified extracted closure.
- Local tarball/package spec execution pinned to the verified artifact.
- npm cache path only if npm can be forced not to re-resolve or fetch outside
  the verified closure.

Acceptance criteria:

- `safe-npx create-example@latest` cannot inspect one version and execute
  another.
- Registry metadata changes between inspection and execution fail closed.
- Tags, ranges, cache entries, dependency versions, and lifecycle scripts cannot
  escape the verified closure.
- Non-interactive mode stops when policy requires a question.
- Similar-name and unusual-shape heuristics remain report-only until validated.

## M4: Execute Mode Alpha

Goal: safely run the intended command only after policy allows the verified
execution closure.

Deliverables:

- Execute mode for the supported v0 spec matrix.
- Approval prompt for interactive mode.
- Explicit inspect-only behavior for `--json` unless an execute path is chosen.
- Local approval cache keyed by artifact digest, command intent, policy version,
  registry source, authority context, and expiry.
- Cache invalidation on digest, policy, registry source, authority context, or
  expiry changes.
- Alpha package distribution for dogfooding.

Acceptance criteria:

- Saying no runs nothing from the package.
- Saying yes runs only from the verified execution closure.
- Approval in one authority context does not silently apply to a materially
  broader context.
- Dogfood runs record latency, unsupported command shapes, policy decisions, and
  bypass pressure.

## M5: Fixture Corpus And Decision Validation

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

Acceptance criteria:

- Every policy rule has pass/fail fixtures.
- Fixture runs prove package code does not execute during inspection.
- Human decision studies show whether reports change stop/continue behavior.
- Agent decision tests show JSON causes stop, explain, approval request, or
  fail-closed behavior.
- Latency, false-positive, and false-negative notes are published.

## M6: Agent, CI, And Early Adoption

Goal: make the tool usable in the places blind package execution is most
dangerous.

Deliverables:

- Agent integration guide for `safe-npx --json` inspect mode.
- CI examples for fail-closed package execution.
- Shell and documentation patterns that nudge humans and agents away from raw
  `npx`.
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
- Release diff mode.
- Broad private registry productization.
- `pnpm dlx`, `yarn dlx`, `bun x`, and other ecosystems.
- Runtime sandbox profiles.
- Package-name dispute or registry governance workflows.

## Milestone Dependency Order

1. M1 blocks all meaningful evidence work.
2. M2 blocks policy, agent JSON, and validation.
3. M3 blocks safe execution.
4. M4 blocks dogfooding and adoption.
5. M5 validates whether the policy and evidence are worth keeping.
6. M6 should start only after inspect mode, JSON, and fail-closed behavior are
   stable enough for external feedback.
