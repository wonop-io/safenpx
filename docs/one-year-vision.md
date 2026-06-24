# One-Year Vision

In one year, `safe-npx` should be the obvious safer wrapper for one-off npm
package execution: it resolves the exact artifact, proves the inspected bytes
are the bytes that run, and gives humans and agents enough evidence to avoid
uninformed execution.

`npx` is useful because it lets a developer run an npm package as a command
without first setting up a project. That convenience is also the danger: a short
command copied from docs, a README, an issue, or an AI-generated answer can
download and execute remote code with broad access to the local machine.

The default prompt asks for trust before it gives useful evidence. `safe-npx`
should change that moment. It should resolve the exact package version, verify
the artifact, inspect what is about to run, apply policy, and only then delegate
to the normal `npx` / `npm exec` path.

The year-one promise is deliberately narrow:

> `safe-npx` is a safety checkpoint before package code from the internet runs.

`safe-npx` has authority over one decision only: whether this exact artifact may
run through the intended execution path now.

Done well, this should make small CLIs in docs, READMEs, and agent skills easier
to try because the risk is visible before code runs.

It should not replace npm, prove that code is safe, or rebuild the JavaScript
package ecosystem. It should make blind remote execution harder to justify and
easier to replace with visible, repeatable evidence.

In plain English, the minimum useful product is: resolve, download, verify,
inspect, explain, ask, then run the intended command or stop.

## Terms

- **One-off executable package:** an npm package run through `npx` / `npm exec`,
  usually without adding it to a project.
- **Artifact:** the exact compressed package download, or tarball, selected for
  execution.
- **Integrity:** identity verification for bytes; not proof of safety.
- **Lifecycle script:** a package install hook npm may run before the requested
  binary.
- **Execution closure:** any package, script, binary, shim, or package-manager
  step that can execute before or during the requested command.
- **Inspect mode:** resolve, verify, inspect, and report without running package
  code.
- **Execute mode:** run only after policy allows the verified execution closure.
- **Typo-squat:** a malicious package named to resemble a trusted package.
- **Fail closed:** stop when evidence, resolution, or policy is insufficient.

## Before And After

The current flow collapses install and execute, then asks for a yes/no decision
with too little context:

```text
npx create-example@latest
Need to install the following packages:
create-example@latest
Ok to proceed? (y)
```

The `safe-npx` flow should preserve the convenience while making the decision
substantive:

```text
safe-npx create-example@latest

Resolved: create-example@3.2.1
Tarball: https://registry.npmjs.org/create-example/-/create-example-3.2.1.tgz
Integrity: sha512 verified
Published: 18 minutes ago
Publisher: example-maintainer
Package size: 412 KB, 86 files
Binaries: create-example
Lifecycle scripts: postinstall
Dependencies: 87 declared nodes

Decision: ask
Reason: very recent release with lifecycle script

Proceed? [y/N]
```

If the user says yes, `safe-npx` runs the same command the user intended, using
the inspected artifact. If the user says no, nothing from the package runs.

For agents and automation, the same decision should be available as stable JSON:

```text
safe-npx --json create-example@latest
```

An agent should not need to scrape terminal text. It should receive structured
artifact identity, evidence, policy status, and next-step requirements so it can
stop, explain the risk, ask the user, or fail closed.

## First Users

The first users are not every npm user. The first users are people and systems
standing at the remote-code execution boundary:

- Developers copying `npx` commands from docs, READMEs, issues, and chat.
- Coding agents asked to run `npx` commands from generated instructions.
- Teams that want a lightweight policy gate before remote executable packages
  run on developer machines or in CI.
- Security-conscious maintainers who want their executable packages to be
  inspectable before users run them.

Adoption depends on speed and clarity. `safe-npx` should feel like the workflow
developers already know, with a better decision point rather than a heavy
security ritual.

## Year-One Shape

By the one-year mark, the project should be a focused, production-quality Rust
CLI with a small set of reliable supporting pieces:

- A wrapper for `npx` and `npm exec` that never runs package code before
  inspection and policy evaluation.
- A resolver and artifact verifier for common npm package specs.
- A static evidence extractor for package metadata and tarball contents.
- A policy engine that returns `allow`, `ask`, `deny`, or `override-required`.
- Human-readable terminal output and stable JSON output.
- A local cache for exact package-version evidence and previous approvals.
- A reproducible fixture corpus for malicious packages, compromised maintainers,
  typo-squats, lifecycle scripts, and agent-driven blind execution.
- Documentation for humans, coding agents, and teams integrating the JSON
  decision format.

Rust is appropriate because the CLI needs predictable startup, careful
filesystem and process boundaries, and a small auditable binary with minimal
runtime surprise.

The central invariant is: every package byte that can execute must be inside the
verified execution closure. Year one must prove this with tests and
implementation design, not aspiration. `safe-npx` must either execute directly
from a verified local closure derived from inspected artifacts, delegate only
through a mechanism that pins npm to that exact closure, or refuse. It must never
inspect `name@latest` and then allow `npm exec` to resolve `latest` again.

The product should be organized around three nouns:

- **Artifact:** the exact bytes about to run.
- **Evidence:** observable facts about those bytes and their provenance.
- **Decision:** the policy outcome before execution.

Everything else should earn its place by strengthening one of those three.

## Evidence V1

The first useful version should focus on evidence that changes the decision
before execution:

- Requested command and full command intent.
- Resolved package name and exact version.
- Registry, tarball URL, integrity metadata, and verification result.
- Publish time, release age, publisher, maintainers, and repository metadata.
- Package size, file count, and unusual package shape.
- Binaries exposed by the package.
- Lifecycle scripts such as `preinstall`, `install`, and `postinstall`.
- Dependency declarations and, once supported, the verified dependency execution
  closure; the UI must label which one is being shown.
- Similar-name and unusual-shape signals, clearly labeled as report-only
  heuristics until fixture-backed validation proves they should affect policy.
- Whether this exact artifact and command have been seen or approved locally
  before, including the authority context in which that happened.

Facts and heuristics must be distinct. Integrity verification is a fact.
Typo-squat similarity is a heuristic. The UI and JSON schema should not blur the
difference.

Static inspection can only report observable pre-execution risks. It cannot
infer package intent or prove benign behavior.

## What It Catches And Does Not Catch

`safe-npx` should be plain about its protection boundary.

It can catch or interrupt tag movement, resolver ambiguity, artifact mismatch,
lifecycle scripts before approval, unsupported specs, package-manager behavior
that would re-resolve, fresh releases, first-seen artifacts, maintainer or
publisher changes, unusual authority deltas, and agent or CI attempts to run
remote executable packages without an approval path.

It does not prove inspected code is benign, runtime behavior is safe after
approval, transitive dependency attacks are covered unless the dependency
execution closure was resolved and verified, filesystem/network/secret access is
contained without a sandbox, or typo-like names and unusual package shapes are
malicious.

## Default Policy V1

The default policy should separate showing evidence from interrupting execution.
The report should be visible whenever useful; prompts should be reserved for
signals strong enough to change behavior.

| Signal | Interactive default | Non-interactive default | Notes |
| --- | --- | --- | --- |
| Integrity mismatch | deny | deny | Fact, not heuristic. |
| Execution closure cannot be verified | deny | deny | Core invariant. |
| Unsupported package spec or resolver ambiguity | deny | deny | No raw `npx` fallback. |
| Lifecycle script present in closure | ask | stop | Especially `preinstall`, `install`, `postinstall`. |
| Very recent publish | ask | stop | Thresholds are experimental and fixture-backed. |
| First-seen exact artifact | visible report | stop unless policy allows | Not automatically scary for humans. |
| Maintainer, publisher, registry, or authority-context delta | ask | stop | Approval cache must include authority context. |
| Similar-name signal | report | report or stop by policy | Heuristic until validated. |
| Large or unusual package shape | report | report or stop by policy | Heuristic until validated. |
| Missing optional heuristic data | allow or ask based on facts | stop only if required evidence is missing | Avoid pretending optional signals are facts. |

The policy engine should explain decisions in terms of evidence, not vibes. If
the tool cannot know enough, automation should stop instead of guessing.
There is no magic safety score in V1. Reports may summarize risk, but decisions
must cite concrete facts or explicitly labeled heuristics.

## Architectural Concepts

The architecture should stay small and auditable. The key boundary is between
inspection and execution.

### 1. Resolver

The resolver turns user intent into exact package coordinates without executing
package code.

Year-one support should start with:

- `name`
- `name@version`
- `name@latest`
- scoped packages such as `@scope/name`

Complex version ranges, workspace context, alternate package managers, peer
dependency behavior, and lockfile-specific semantics should be delegated to
existing package-manager behavior where possible rather than reimplemented
prematurely.

Unsupported specs should produce a clear refusal that says what was unsupported,
whether anything was downloaded, and the nearest supported command form. They
should never silently fall back to raw `npx`.

### 2. Artifact Verifier

The artifact verifier downloads package tarballs and checks that the bytes match
the expected integrity metadata.

The important unit is not a package name in general. It is a specific package
version, resolved at a specific time, with a specific tarball digest. All later
evidence should attach to that artifact identity.

### 3. Evidence Extractor

The evidence extractor inspects package metadata and contents without running
package code.

It should collect deterministic facts first: `package.json`, bins, lifecycle
scripts, file count, package size, dependency declarations, and registry
metadata. Heuristic signals such as obfuscation, generated code, typo-squat
similarity, and unusual package shape should be labeled as heuristics.

### 4. Execution Authority Context

The hardest risk is not only "what package is this?" It is also "what authority
will this process have if it runs?"

Year one should report authority without leaking private data:

- Mode: interactive or non-interactive, with agent and CI as named
  non-interactive callers.
- Current working directory, reduced to a privacy-preserving project label when
  possible.
- Environment exposure summary: counts and categories, not secret values.
- Whether likely secret-bearing variables are present.
- Filesystem read/write exposure at normal process privileges.
- Network availability at normal process privileges.
- Registry source and whether private registry config was used.
- Package binary and forwarded arguments.
- Lifecycle script status.

Approval cache keys must include artifact digest, command intent, policy version,
expiry, registry source, and authority context. An approval in one authority
context should not silently apply to a materially broader one.

Runtime permissions and sandbox profiles should remain an explicit research
track. Without them, `safe-npx` is an evidence and policy gate, not containment.

### 5. Policy Engine

The policy engine turns evidence into a decision:

- `allow`: proceed without interaction.
- `ask`: show evidence and require approval.
- `deny`: stop execution.
- `override-required`: allow only with an explicit, logged override.

Policy should work locally first. Organization-managed policy can follow once
the local loop is useful and trusted.

### 6. Inspect And Execute Contract

The CLI has two contracts.

Inspect mode resolves, downloads, verifies, extracts evidence, and returns a
decision without running package code. JSON output is inspect mode unless an
explicit execute flag or command path is used.

Execute mode may run only after policy allows the verified execution closure.
That closure includes the root artifact, selected binary, generated shims,
dependency packages needed by normal execution, and lifecycle scripts that npm
would run. If `safe-npx` cannot determine or pin that closure, it must refuse.

The first execution spike should prefer direct execution from a verified
extracted closure. Delegating through npm is acceptable only if tests prove npm
cannot re-resolve tags, ranges, registry metadata, cache entries, or dependency
versions outside the inspected closure.

Acceptance examples: pass when `safe-npx create-example@latest` resolves
`3.2.1`, verifies digest `X`, verifies closure `C`, and executes only files from
`C`; fail closed when registry metadata changes, npm would re-resolve a
tag/range/URL, dependency installation would run unapproved lifecycle scripts, or
the cache cannot prove provenance.

### 7. CLI Contract V1

The first stable surface should be explicit enough for humans, agents, and CI:
`safe-npx <pkg-spec> [-- <args>]` preserves forwarded package arguments;
`safe-npx --json <pkg-spec> [-- <args>]` emits a machine-readable decision; no
package code, lifecycle script, or binary runs before the decision; human mode
may prompt; agent and CI modes stop by default when policy requires a question;
exit codes distinguish successful execution, ask-required, deny, unsupported
input, inspection error, and delegated execution failure. Approval cache entries
attach to artifact digest, package coordinates, command intent, and policy
version, and invalidate on digest, material metadata, maintainer or publisher
context, policy, or expiry changes.

### 8. Agent Protocol

`safe-npx` should provide a stable agent-facing contract from the beginning.

The protocol should include `artifact`, `command_intent`, `facts`, `heuristics`,
`decision`, `reasons`, `required_next_action`, and `exit_code`.

The first integration target is simple: an agent wants to run `npx`, calls
`safe-npx --json`, and stops unless the policy result permits execution.

A canonical agent scenario is a `SKILL.md`, rules file, or generated instruction
that tells an agent to run `npx some-tool@latest`. `safe-npx` should make that
remote execution request explicit before the agent acts.

## Validation

The project should validate that it changes real decisions, not only that it
prints more data.

Year-one validation should include:

- Artifact invariant tests proving inspected digest equals executed digest.
- Inspection safety tests proving package code and lifecycle scripts do not run
  before approval.
- Human decision studies showing whether developers stop, ask, or continue
  differently after seeing the report.
- Agent decision tests showing that JSON output causes agents to stop, explain,
  request approval, or fail closed.
- A fixture corpus with benign CLIs, lifecycle-script packages, fresh releases,
  compromised-maintainer simulations, typo-like names, obfuscated root packages,
  and dependency-declaration traps.
- Pass/fail examples for every default policy rule.
- `npx` compatibility and refusal matrix for supported specs, unsupported specs,
  forwarded args, bins, lifecycle scripts, cache behavior, and tag moves.
- Dependency confusion and private-registry precedence fixtures using `.npmrc`
  and scoped registries.
- Real incident replay fixtures for compromised maintainers, malicious updates,
  typo-like names, and lifecycle-script abuse.
- Command provenance tests for docs, READMEs, generated agent instructions, and
  `SKILL.md` paths.
- Privacy tests proving reports and JSON never expose secrets or registry tokens.
- Latency measurements for clean-cache and warm-cache runs.
- False-positive review for prompt fatigue.
- False-negative review for missed suspicious fixtures.

If `safe-npx` is too slow or too noisy, users will bypass it. Speed and signal
quality are product requirements, not polish.

## 30 / 60 / 90 Days

The first 90 days should prove the local decision loop.

### 30 Days

- Prove the core loop, not the whole product.
- Resolve `name`, `name@version`, `name@latest`, and scoped package specs.
- Download tarballs and verify integrity.
- Decide the execution mechanism for exact inspected bytes.
- Prove in tests that the inspected tarball digest is the executed artifact.
- Prove in tests that lifecycle scripts do not run during inspection.
- Extract root package evidence from registry metadata, tarball metadata, and
  `package.json`.
- Emit human report plus minimal JSON using the future schema shape.
- Implement hardcoded policy v0 using the threshold table.
- Make unsupported specs fail closed with useful messages.
- Add fixture tests for integrity mismatch, lifecycle scripts, recent publish,
  first-seen artifact, typo-like names, and resolver ambiguity.
- Publish a terminal-first demo comparing the default `npx` prompt with
  `safe-npx`.

### 60 Days

- Stabilize the `--json` schema around the minimum agent contract.
- Add a local policy file.
- Add local cache records for exact artifact inspections and approvals.
- Invalidate cache approvals on digest, policy, and material metadata changes.
- Expand the fixture corpus.
- Document human, agent, and CI modes with exit-code behavior.
- Add agent integration docs showing stop-by-default behavior.
- Add dependency graph summary without pretending to fully reimplement npm.
- Start feedback with agent-tool builders and package-security maintainers.

### 90 Days

- Add release diff mode for previously seen package versions.
- Add CI mode for fail-closed execution.
- Add `.npmrc` smoke support for auth, scoped registries, and precedence
  reporting; do not build a private package platform.
- Publish JSON schema v0.1.
- Publish the fixture corpus as a benchmark.
- Publish latency, false-positive, and false-negative notes.
- Recruit early adopters using `safe-npx` in human, agent, or CI workflows.
- Validate that users or agents changed decisions because of evidence.
- Treat release diff mode as a stretch unless the core artifact invariant,
  policy table, JSON schema, and validation loop are already stable.
- Decide whether hosted audit records are justified by reuse and user demand.

## Issue-Shaped Milestones

Early work should be tracked as implementation issues, not themes:

- Resolver v0: supported spec matrix, registry config handling, unsupported
  failures, fixtures.
- Artifact verifier v0: tarball download, integrity, digest identity.
- Execution closure spike: selected execute mechanism, dependency/lifecycle
  closure rules, invariant tests.
- Inspect mode v0: extract evidence without running package code.
- Execute mode v0: refusal matrix for cases where closure cannot be pinned.
- Policy v0: experimental thresholds with fixture-backed pass/fail examples.
- JSON schema v0: artifact, command intent, authority context, facts,
  heuristics, decision, reasons, required next action, and exit code.
- Report v0: facts, heuristics, decision, command authority, and explicit
  catches/does-not-catch language.
- Cache trust model v0: approval keys, expiry, authority context, invalidation.
- Fixture manifest v0: package, digest, closure, expected signals, expected
  result.
- Alpha dogfood release: packaged CLI used locally on real `npx` commands.

## Non-Goals

`safe-npx` should be explicit about what it is not:

- Not a replacement for npm, pnpm, Yarn, Bun, or package registries.
- Not proof that a package is safe, a magic safety score, or a containment system
  when no sandbox is active.
- Not a full transitive dependency attack analyzer in V1.
- Not a public audit authority before the local CLI proves useful.
- Not a reimplementation of every package-manager resolution rule.
- Not a prompt-only convention; it is a wrapper with an enforceable decision.
- Not dependent on paid or hosted audits for the local safety loop.
- Not willing to execute when inspected bytes may differ from executed bytes, or
  silently downgrade failed inspection to raw `npx`.
- Not broad private registry support beyond smoke support for common `.npmrc`
  workflows.

These constraints keep the project credible. The transcript points at broader
npm ecosystem problems, including publishing, revocation, name ownership,
private distribution, and audit economics. `safe-npx` chooses the `npx`
execution boundary first because it is narrow, painful, and immediately useful.

## Design Principles

- Evidence before execution; never execute if inspected bytes may differ.
- Judge the exact version, not just the package name; never present integrity as
  safety.
- Make authority visible and treat maintainer or publisher compromise seriously.
- Prefer transparent facts over opaque scores; label heuristics as heuristics.
- Stay useful without hosted services; keep the execution path small.
- Make human output readable, JSON stable, and automation stop when uncertain.
- Use reproducible fixtures; defer platform expansion until the local loop works.

## Open Questions

- How much dependency graph resolution is honest without rebuilding npm?
- Which signals block, ask, or only report?
- What latency budget keeps developers from bypassing the tool?
- What authority model is useful before sandboxing exists?
- How should approvals expire when publisher or package metadata changes?
- What should the minimum viable agent protocol include?
- How is private package metadata protected if shared records exist?
- Which package-manager and agent integrations come first after npm works?

These questions should stay visible. The project should move quickly without
pretending the trust model is simpler than it is.
