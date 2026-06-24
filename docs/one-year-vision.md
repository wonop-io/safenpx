# One-Year Vision

In one year, `safe-npx` should be the obvious better prompt for running package
code from the internet.

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

Done well, this should make one-off executable npm packages more usable, not
merely more frightening. Small CLIs in docs, READMEs, and agent skills should be
easier to try because the risk is visible before code runs.

It should not replace npm, prove that code is safe, or rebuild the JavaScript
package ecosystem. It should make blind remote execution harder to justify and
easier to replace with visible, repeatable evidence.

In plain English, the minimum useful product is: resolve, download, verify,
inspect, explain, ask, then run the intended command or stop.

## Terms

- **One-off executable package:** an npm package run as a command through `npx`
  or `npm exec`, usually without adding it to a project.
- **Artifact:** the exact compressed package download, or tarball, selected for
  execution.
- **Integrity:** identity verification for bytes. It means the download matches
  what the registry said, not that the code is harmless.
- **Lifecycle script:** a package-defined install hook that npm may run before
  the requested binary.
- **Typo-squat:** a malicious package with a name that looks like a popular or
  trusted package.
- **Fail closed:** stop instead of guessing when evidence, resolution, or policy
  is insufficient.

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

The central invariant is: the bytes inspected are the bytes executed. If
`safe-npx` cannot prove that, it must not delegate execution.

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
- Direct dependency count, plus an optional dependency summary only when it can
  be produced without pretending to fully model npm resolution.
- Similar package names and typo-squat signals.
- Whether this exact artifact has been seen or approved locally before.

Facts and heuristics must be distinct. Integrity verification is a fact.
Typo-squat similarity is a heuristic. The UI and JSON schema should not blur the
difference.

Static inspection can only report observable pre-execution risks. It cannot
infer package intent or prove benign behavior.

## Default Policy V1

The default policy should separate showing evidence from interrupting execution.
The report should be visible whenever useful; prompts should be reserved for
signals strong enough to change behavior.

- Deny on integrity mismatch.
- Deny when registry metadata and downloaded artifact identity disagree.
- Ask on lifecycle scripts.
- Ask on very recent publishes, for example packages published within the last
  24 hours.
- Ask on first-seen exact artifacts in human mode.
- Ask when a package name looks similar to a better-known package.
- Ask on unusual package shape, high file count, large tarball, or unexpectedly
  large dependency surface.
- Stop by default for agent and CI modes when evidence is missing, unsupported,
  or inconclusive unless policy explicitly allows continuation.

The policy engine should explain decisions in terms of evidence, not vibes. If
the tool cannot know enough, automation should stop instead of guessing.

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

Year one should make that authority visible even if full sandboxing is deferred:

- Mode: `human`, `agent`, or `ci`.
- Current working directory.
- Environment-variable exposure summary, including whether secrets may be
  present.
- Filesystem read/write exposure at normal process privileges.
- Network availability at normal process privileges.
- Package binary and forwarded arguments.
- Whether lifecycle scripts would run.

Runtime permissions and sandbox profiles should remain an explicit research
track. Without them, `safe-npx` is an evidence and policy gate, not a complete
containment system.

### 5. Policy Engine

The policy engine turns evidence into a decision:

- `allow`: proceed without interaction.
- `ask`: show evidence and require approval.
- `deny`: stop execution.
- `override-required`: allow only with an explicit, logged override.

Policy should work locally first. Organization-managed policy can follow once
the local loop is useful and trusted.

### 6. Execution Delegator

The execution delegator runs the original command only after resolution,
verification, evidence extraction, and policy evaluation have completed.

This component should stay small. Everything before execution should be
inspectable. Everything after approval should be clearly delegated to the normal
package execution path.

The delegator must preserve the artifact invariant. It should either execute the
exact artifact inspected or fail with a clear explanation. It must never inspect
one artifact and then let `npm exec` resolve a different one.

### 7. CLI Contract V1

The first stable surface should be explicit enough for humans, agents, and CI:

- `safe-npx <pkg-spec> [-- <args>]` preserves forwarded package arguments.
- `safe-npx --json <pkg-spec> [-- <args>]` emits a machine-readable decision.
- Package code, lifecycle scripts, and package binaries never run before the
  decision.
- Human mode may prompt.
- Agent and CI modes stop by default when policy requires a question.
- Exit codes distinguish successful execution, ask-required, deny, unsupported
  input, inspection error, and delegated execution failure.
- Approval cache entries attach to artifact digest, package coordinates, command
  intent, and policy version.
- Approvals invalidate when the digest changes, package metadata materially
  changes, maintainer or publisher context changes, policy changes, or an expiry
  is reached.

### 8. Agent Protocol

`safe-npx` should provide a stable agent-facing contract from the beginning.

The protocol should include:

- `artifact`
- `command_intent`
- `facts`
- `heuristics`
- `decision`
- `reasons`
- `required_next_action`
- `exit_code`

The first integration target is simple: an agent wants to run `npx`, calls
`safe-npx --json`, and stops unless the policy result permits execution.

A canonical agent scenario is a `SKILL.md`, rules file, or generated instruction
that tells an agent to run `npx some-tool@latest`. `safe-npx` should make that
remote execution request explicit before the agent acts.

## Validation

The project should validate that it changes real decisions, not only that it
prints more data.

Year-one validation should include:

- A fixture corpus with benign CLI packages, lifecycle-script packages,
  typo-squat examples, maintainer-takeover simulations, obfuscated packages,
  fresh releases, and suspicious transitive dependencies.
- Latency measurements on clean cache and warm cache runs, with targets tight
  enough that developers do not bypass the tool.
- Tests showing that package code is not executed during inspection.
- Human decision tests: does the report help a developer choose stop, ask, or
  continue?
- Agent decision tests: does the JSON make an agent stop, explain, and ask?
- False-positive review: which warnings are too noisy for real workflows?
- False-negative review: which malicious or suspicious fixtures are missed?

If `safe-npx` is too slow or too noisy, users will bypass it. Speed and signal
quality are product requirements, not polish.

## 30 / 60 / 90 Days

The first 90 days should prove the local decision loop.

### 30 Days

- Resolve common npm package specs to exact package versions.
- Download tarballs and verify integrity.
- Extract root artifact evidence from registry metadata and `package.json`.
- Print a human-readable report before execution.
- Add a minimal hardcoded policy contract and block execution until approval or
  policy allows it.
- Add minimal `--json` output and exit codes for agents.
- Add fixture tests for integrity mismatch, lifecycle scripts, recent publish,
  and typo-like package names.
- Prove in tests that the inspected tarball digest is the executed artifact.
- Prove in tests that lifecycle scripts do not run during inspection.
- Make unsupported specs fail closed with useful messages.
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
- Add private registry smoke support for `.npmrc`-based workflows, without making
  private registry productization central.
- Publish JSON schema v0.1.
- Publish the fixture corpus as a benchmark.
- Publish latency, false-positive, and false-negative notes from real runs.
- Recruit early adopters using `safe-npx` in human, agent, or CI workflows.
- Validate that users or agents changed decisions because of evidence.
- Decide whether a hosted audit registry is justified by repeated evidence reuse
  and user demand.

## Non-Goals

`safe-npx` should be explicit about what it is not:

- It is not a replacement for npm, pnpm, Yarn, Bun, or package registries.
- It does not guarantee that a package is safe.
- It does not fully sandbox arbitrary JavaScript in year one.
- It does not become a public audit authority before the local CLI proves useful.
- It does not reimplement every package-manager resolution rule unless that is
  required to make the execution decision honest.
- It does not treat package names or maintainer reputation as enough.
- It does not silently downgrade from failed inspection to raw `npx`.
- It does not execute when inspected bytes may differ from executed bytes.
- It does not depend on paid or hosted audits to make the local safety loop
  useful.

These constraints keep the project credible. The transcript points at broader
npm ecosystem problems, including publishing, revocation, name ownership,
private distribution, and audit economics. `safe-npx` chooses the `npx`
execution boundary first because it is narrow, painful, and immediately useful.

## Deferred Platform Bet

If the local CLI proves that evidence changes decisions, the project can grow
into a broader trust layer for tiny executable packages and agent-invoked tools.

Promising later directions include:

- Hosted artifact report pages for exact package versions.
- Reusable audit records with expiration and reviewer metadata.
- Portable third-party audit attestations for exact package versions.
- Author-facing audit badges for executable packages.
- Release diff reports for package authors and users.
- Package-name similarity and dispute evidence for registry operators.
- Private registry and package-bucket policy support for internal executable
  tools.
- Agent instruction scanning for `npx` commands in docs, rules, and `SKILL.md`
  files.
- Runtime sandbox or permission profiles.
- Support for adjacent commands such as `pnpm dlx`, `yarn dlx`, `bun x`, and
  eventually other ecosystems.

These ideas are real, but they should not dilute the year-one test:

> Can `safe-npx` make the old `npx` yes/no prompt impossible to respect without
> first asking, "with what evidence?"

## Design Principles

- Evidence before execution.
- Never execute when inspected bytes may differ from executed bytes.
- Never silently downgrade from inspection to raw `npx`.
- Never present integrity as safety.
- Judge the exact version being run, not just the package name.
- Make package authority visible.
- Treat maintainer and publisher compromise as first-class threats.
- Prefer transparent facts over opaque scores.
- Label heuristics as heuristics.
- Stay useful without a hosted service.
- Never require a hosted trust service for the local safety loop.
- Keep the execution delegator small.
- Make human output readable and JSON output stable.
- Make agents and CI stop when uncertain.
- Use reproducible fixtures for every important threat.
- Defer platform expansion until the local decision loop works.

## Open Questions

- How much dependency graph resolution can be done honestly without rebuilding a
  package manager?
- Which signals are strong enough to block by default, and which should only
  escalate to `ask`?
- What latency budget keeps developers from bypassing the tool?
- What is the minimum useful authority model before sandboxing exists?
- How should local approvals expire when publishers, maintainers, or package
  metadata change?
- What should the minimum viable agent protocol include?
- How should private package metadata be protected if a shared registry is used?
- Which package-manager and agent integrations should come first after the npm
  path works?

These questions should stay visible. The project should move quickly, but it
should not pretend the trust model is simpler than it is.
