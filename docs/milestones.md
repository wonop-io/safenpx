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
5. Decide with the canonical enum: allow, ask, deny, unsupported,
   inspection_error, or execution_refused.
6. Execute only if the inspected bytes are the bytes that will run.

Execution closure means every local artifact that can execute as part of the
command: the root tarball, selected binary, generated shim, installed
dependencies, lifecycle scripts, package-manager helpers, and any resolved files
needed before the command starts. Dependency declarations are not an execution
closure; they are only evidence until the resolver proves and verifies them.
For example, `create-foo@1.0.0` may run its own bin, an npm-generated shim, and
dependency install scripts before the requested command starts; all of that is
inside the closure. Verifying the root tarball is necessary, but not sufficient
to execute.

Fail closed means stop rather than falling back to raw `npx`. M3 is useful as a
no-run inspection tool even if M2 proves that safe execution must remain
inspect-only for the alpha.

Decision vocabulary:

- `allow`: evidence and policy permit the requested action.
- `ask`: a human decision is required before execution.
- `deny`: a known proof failure or unsafe condition was found.
- `unsupported`: the command shape is outside the current implementation.
- `inspection_error`: evidence could not be collected reliably.
- `execution_refused`: inspection succeeded, but safe execution cannot be
  proven for this package topology.

Reason vocabulary starts with `integrity_mismatch`, `unsupported_spec`,
`malformed_spec`, `registry_error`, `ambiguous_bin`, `missing_bin`,
`lifecycle_script_present`, `unsupported_closure`, `metadata_changed`,
`policy_requires_interaction`, and `non_interactive_stop`.

## Milestone Principles

- Prove the execution boundary before expanding the product surface.
- Keep the first useful path narrow: common npm package specs, root evidence,
  inspect mode, JSON output, exact-version execution, and clear refusal.
- Treat v0 policy thresholds as provisional constants until fixtures and
  dogfooding prove they change decisions.
- Start fixtures with the resolver instead of treating QA as a late phase.
- Keep hosted audit records, broad private registry support, and alternate
  package managers out of the critical path.
- Keep private/scoped registry work inspect-only until the public npm proof
  path works.
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
- Forwarded package arguments after `--`, for example
  `safe-npx create-example@1.2.3 -- --template react`.

Supported after the tag-race proof:

- `name@latest`
- `@scope/name@latest`

Explicitly unsupported in v0:

- Unversioned names.
- Version ranges other than `latest`.
- Git URLs, local paths, tarball URLs, aliases, and multiple package specs.
- `npm exec --package`, `-c`, and package-manager-specific variants.

V0 intentionally does not support the common `npx name` form. Floating intent
is useful, but exact versions are the first path where the artifact proof is
small enough to make honest progress.

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
- First runnable package class: exact version, one selected bin, no lifecycle
  scripts, no dependency installation unless the dependency closure is fully
  verified, no npm re-resolution, and macOS/Linux first.
- M2 must choose the first execution path: root-only/no-deps runnable subset,
  fully verified dependency closure, or inspect-only alpha.
- Deny dependency lifecycle scripts by default until they are part of the
  verified closure.
- Treat dependency declarations as evidence only unless dependencies are
  resolved, downloaded, integrity-checked, and included in the closure.
- Prefer direct execution from the verified extracted artifact for the first
  proof. Package-manager delegation comes later only if it preserves byte
  identity.
- Add `latest` only after tag-move and refetch races fail closed.

Deliverables:

- Execution-closure design note comparing direct execution, pinned local
  tarball execution, and package-manager delegation.
- Decision record choosing `direct_extract`, `pinned_delegation`, or
  `inspect_only_alpha` for first alpha execution. M5 may not start until M2
  chooses one with fixture evidence.
- No-package-code-ran canary harness using sentinel files, environment markers,
  and blocked network attempts.
- Race matrix for resolution time versus execution time.
- Bin selection rules and refusal behavior for ambiguous or missing bins.
- Bin fixtures for single bin, multiple bins without explicit selection, missing
  bin, scoped package bin, and forwarded args preserved exactly.
- Registry and `.npmrc` precedence tests for public npm and scoped registry
  inspect-only smoke cases. npm may choose different registries based on scope,
  directory, or config, so inspection and execution must agree on the same
  registry source.
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
- If full dependency closure cannot be proven, execution returns
  `execution_refused` with reason `unsupported_closure`.
- If the executable subset is too narrow for useful alpha execution, the alpha
  ships as inspect-only rather than weakening the invariant.

Decision:

- M2 chooses `direct_extract` for the first alpha execution path. See
  `docs/m2-execution-decision.md`.
- The first runnable class is exact-version, verified root artifact, one
  deterministic selected root bin, no lifecycle scripts, no dependencies unless
  the full dependency closure is proven, and no package-manager delegation.
- Unsupported topologies remain inspect-only or return `execution_refused`;
  unproven dependency closure returns reason `unsupported_closure`.

## M3: Inspect Evidence And JSON V0

Goal: produce useful human and machine evidence before any package code can run.

Status: complete. Closeout evidence is recorded in
`work-trace/65/m3-closeout-evidence.md`; GitHub milestone #4 is closed with
zero open issues.

Deliverables:

- Inspect mode that resolves, downloads, verifies, extracts evidence, and stops.
- Static extraction from registry metadata, tarball metadata, and `package.json`.
- Human report separating facts, provisional heuristics, decision, and authority
  context.
- Versioned JSON schema with deterministic ordering and golden fixture outputs.
- Compatibility rule: additive fields are allowed in `0.x`, enum additions
  require a schema bump, and enum semantic changes require a migration note.
- JSON fields for `schema_version`, `artifact`, `command_intent`,
  `source_context`, `authority_context`, `facts`, `heuristics`,
  `external_evidence`, `attestations`, `release_diff`, `decision`, `reasons`,
  `required_next_action`, `execution`, and `exit_code`.
- Nullable reserved fields for `external_evidence`, `attestations`, and
  `release_diff`; V0 must not implement hosted audits or release diffs.
- Decision receipt fields for local/shareable records: artifact digest, command
  intent, evidence summary, policy version, timestamp, and redaction metadata.
  M3 defines the shape; M4 defines semantics; M5 may cache receipts only after
  authority-context tests pass.
- Privacy and redaction rules for reports and JSON.

Evidence v0:

- Requested command, selected bin, and forwarded args.
- Source context: manual terminal, README/docs snippet, agent skill, CI, or
  unknown. In V0 this is caller-declared metadata or `unknown`, not magic
  detection.
- Resolved package identity and integrity verification.
- Registry source and publish time.
- Publisher, maintainers, repository, license, and provenance fields when
  available.
- Package size, file count, binaries, and lifecycle scripts.
- Dependency declarations, clearly labeled as declarations unless included in
  the verified execution closure.
- Similar-name and unusual-shape heuristics, meaning warning signs that are not
  proof, as report-only signals.

JSON enums:

- `decision`: `allow`, `ask`, `deny`, `unsupported`, `inspection_error`,
  `execution_refused`.
- `required_next_action`: `none`, `ask_user`, `retry_narrower_command`,
  `inspect_only`, `explicit_override`, `unsupported`.
- `mode`: `inspect`, `execute`.
- `execution`: null in inspect mode; populated only when the command can run
  from the verified closure without re-resolution.

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
- Authority context examples include local terminal versus CI, trusted project
  directory versus temp directory, public npm versus scoped registry, and manual
  user versus coding agent. It describes ambient process authority; it is not a
  sandbox.
- Redacted display output remains separate from hashed or canonicalized identity
  fields used for cache keys and receipts.
- The report plainly states what `safe-npx` catches and does not catch.
- Provisional inspect latency budgets are measured: cold public-package inspect
  under five seconds and warm inspect under one second.

## M4: Provisional Policy V0 And Exit Semantics

Goal: turn evidence into predictable allow, ask, deny, and stop outcomes.

Policy v0 is intentionally provisional. Thresholds should be constants that are
easy to change after fixture and dogfood feedback.

Status: planned. GitHub issue #6 is the parent index; implementation work is
split into issues #66 through #76.

Initial thresholds:

- Recent publish warning: package version published within 24 hours, because
  very new releases have had less time for users, maintainers, or automated
  systems to notice compromise.
- Large package warning: tarball larger than 5 MB.
- Large file-count warning: more than 500 files.
- Lifecycle script: ask in interactive mode, stop in non-interactive mode.
- Integrity mismatch or resolver ambiguity: deny.
- Unsupported closure: execution_refused.
- Hard denials are proof failures. Heuristic warnings should not become denials
  until validated.

Deliverables:

- Interactive and non-interactive modes.
- Exit codes for successful inspection, ask-required, deny, unsupported input,
  inspection error, execution refused, and delegated execution failure.
- Example outcomes for `allow`, `ask`, `deny`, `unsupported`,
  `inspection_error`, `execution_refused`, and non-interactive stop.
- Provisional policy fixtures covering every threshold and fail-closed path.
- Agent-readable JSON decision semantics for both interactive and
  non-interactive mode.

Ticket map: #66 policy model, #67 thresholds, #68 interactive ask, #69
non-interactive stop, #70 exit codes, #71 next actions, #72 fixtures and
goldens, #73 heuristic guardrails, #74 integration coverage, #75 docs, and #76
closeout audit.

Acceptance criteria:

- Non-interactive mode stops when policy requires a question.
- Similar-name and unusual-shape heuristics remain report-only until validated.
- Every policy rule has pass/fail fixtures.
- Fixture output explains whether the user can retry with a narrower command,
  inspect-only mode, or an explicit override.

Exit code mapping:

- `0`: successful inspection or execution.
- `10`: ask required.
- `11`: denied.
- `12`: unsupported input.
- `13`: inspection error.
- `14`: execution refused.
- `15`: delegated execution failed.

## M5: Execute Mode Alpha

Goal: safely run the intended command only after policy allows the verified
execution closure.

Deliverables:

- Execute mode for exact-version specs that pass the closure proof.
- Later support for `latest` after tag-race fixtures pass.
- Approval prompt for interactive mode.
- Explicit inspect-only behavior for `--json` unless `safe-npx execute` is
  chosen. Agent execution must carry the prior artifact identity forward instead
  of triggering a second resolution.
- Local approval cache keyed by artifact digest, command intent, policy version,
  registry source, authority context, and expiry.
- Cache invalidation on digest, policy, registry source, authority context, or
  expiry changes.
- Compatibility matrix showing common `npx` commands that alpha supports,
  refuses, or intentionally does not emulate.
- Alpha package distribution for dogfooding.
- Install path documented for one package manager or binary release, with
  versioned upgrade and uninstall notes.
- `--help`, shell completion, or manpage coverage for the alpha CLI surface.

Acceptance criteria:

- Saying no runs nothing from the package.
- Saying yes runs only from the verified execution closure.
- Approval in one authority context does not silently apply to a materially
  broader context.
- `latest` cannot inspect one version and execute another.
- Dogfood runs record latency, unsupported command shapes, policy decisions,
  bypass pressure, and workaround behavior. Bypass pressure means how often
  people feel tempted to call raw `npx` because `safe-npx` blocked them.
- Approval cache starts experimental or disabled until authority-context golden
  tests pass.

## M6: Expanded Fixture Corpus And Decision Validation

Goal: prove that evidence changes behavior without creating unacceptable noise.

Deliverables:

- Fixture manifest format with package, version, registry, digest,
  source_context, authority_context, command_intent, selected_bin,
  forwarded_args, lifecycle-script flag, dependency-closure status, expected
  evidence, expected reasons, expected exit code, and whether execution is
  allowed.
- Benign CLI fixtures.
- Lifecycle-script fixtures.
- Fresh-release fixtures.
- Maintainer-compromise simulations.
- Typo-like name fixtures.
- Obfuscated root package fixtures.
- Dependency-declaration and dependency-confusion fixtures.
- Cache/tag-move fixtures.
- Real incident replay fixtures where legally and ethically usable.
- Remote execution safety benchmark packaging so agent vendors, package
  managers, and security tools can run the corpus.
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
- Go/no-go recommendation decides whether the alpha remains inspect-only or
  expands execute support.

## M7: Agent, CI, Author, And Early Adoption

Goal: make the tool usable in the places blind package execution is most
dangerous.

Deliverables:

- Agent integration guide for `safe-npx --json` inspect mode.
- CI examples for fail-closed package execution.
- Minimal agent-instruction examples that replace `npx` commands from
  `SKILL.md`-style instructions with `safe-npx --json` approval flow.
- Copy-paste replacement snippets for READMEs, agent skills, docs pages, and CI
  scripts.
- Package-author README snippet for inspect-first replacement and `ask` handling.
- Optional `safe-npx suggest` or `safe-npx doctor` concept for scanning docs,
  CI, and agent instruction files for raw `npx` commands.
- Author guidance for safe-npx-friendly CLI packages: clear bins, stable
  releases, minimal lifecycle scripts, provenance fields, and readable metadata.
- Terminal-first demo comparing raw `npx` with `safe-npx`.
- Early adopter feedback from developers, agent-tool builders, package-security
  maintainers, and small CLI authors.

Acceptance criteria:

- Agents can render an approval request without scraping terminal text.
- Agent and CI docs explain how to request or pin supported `schema_version`
  values.
- CI can fail closed on deny, unsupported, inspection error, or ask-required.
- Early adopters can install and run the alpha on real commands.
- Feedback identifies whether hosted audit records, private registry support,
  or release diffing are justified next.
- The project is framed as agent-safety infrastructure: machine-readable stop
  signs before agents execute internet code.

## Deferred Until Core Proof Works

These are real opportunities, but not core milestone blockers:

- Hosted audit records and public artifact pages.
- Third-party audit attestations.
- Author-facing audit badges.
- Release diff mode beyond placeholder fields.
- Broad private registry productization.
- Windows execution semantics beyond smoke or experimental support.
- `pnpm dlx`, `yarn dlx`, `bun x`, and other ecosystems.
- Runtime sandbox profiles.
- Package author preview mode for local tarballs.
- Shell/browser integrations that detect raw `npx` snippets.
- Organization policy templates such as agent strict, developer balanced, CI
  deny lifecycle, and private registry only.
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
