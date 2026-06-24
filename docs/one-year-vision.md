# One-Year Vision

In one year, `safe-npx` should be the default evidence layer between developer
intent and remote package execution.

The project should not try to become a replacement for npm, pnpm, Yarn, Bun, or
agent runtimes. Its role is narrower and more durable: resolve the exact code
that is about to run, collect the evidence needed to judge it, apply local or
organizational policy, and make the decision visible before execution.

The long-term bet is that package execution is becoming an agent boundary. Human
developers already run `npx` commands from documentation and chat answers. Coding
agents will do the same, faster and with less hesitation. The old prompt asks
for trust at the moment when the user has the least context. `safe-npx` should
turn that moment into a structured decision.

## What It Should Become

`safe-npx` should become a small family of interoperable pieces:

- A Rust CLI that wraps `npx` and `npm exec` with evidence-before-execution.
- A stable JSON decision format that agents, CI systems, and IDEs can consume.
- A package-version audit registry that stores reusable evidence for exact npm
  artifacts.
- A policy engine that lets users and organizations decide what is allowed,
  denied, or escalated.
- A reproducible public test corpus covering malicious packages, compromised
  maintainers, typo-squats, lifecycle scripts, and agent-driven blind execution.
- Documentation for package managers, agent vendors, security teams, and open
  source maintainers.

The project should feel boring in the best way: predictable, auditable,
scriptable, and hard to bypass accidentally.

## User Experience

The core command should still feel like the workflow developers already know:

```text
safe-npx create-example@latest
```

Before remote code runs, the user should see:

- The exact package and version resolved from the requested spec.
- Artifact integrity and whether it was verified.
- Release age and freshness signals.
- Maintainer and publisher signals.
- Binaries and lifecycle scripts.
- Dependency graph size and notable dependency findings.
- Whether similar package names look suspicious.
- Whether the package-version has an audit record.
- A recommendation produced from transparent risk signals.
- The policy decision: allow, ask, deny, or require override.

For agents and automation, the same decision should be available as stable JSON:

```text
safe-npx --json create-example@latest
```

The JSON output should be treated as a protocol. An agent should be able to stop
before execution, explain the evidence to the user, ask for approval, or fail
closed according to local policy.

## Architectural Concepts

The architecture should be built around exact artifacts, explicit evidence, and
separable trust decisions.

### 1. Resolver

The resolver turns user intent into exact package coordinates.

It should understand package specs such as `name@latest`, scoped packages,
version ranges, dist-tags, direct versions, and the eventual package-manager
context. Its output should always identify an exact package version and the
registry metadata used to reach that answer.

The resolver must not execute package code.

### 2. Artifact Verifier

The artifact verifier downloads package tarballs and checks that the bytes match
the expected integrity metadata.

The important unit is not "the package name" in general. It is a specific
package version, resolved at a specific time, with a specific tarball digest. All
later evidence should attach to that artifact identity.

### 3. Static Evidence Extractor

The evidence extractor inspects package metadata and contents without running
package code.

It should collect signals such as:

- `package.json` metadata.
- Binaries exposed by the package.
- Lifecycle scripts.
- Files, size, and package shape.
- Bundled or generated code indicators.
- Repository, maintainer, and publisher metadata.
- Dependency declarations.
- Name similarity and typo-squat signals.

This layer should prefer deterministic checks over opaque scoring. When a signal
is heuristic, it should say so.

### 4. Dependency Graph Builder

The dependency graph builder resolves what would enter the execution environment
if the command continued.

It should track integrity for dependency nodes wherever the package ecosystem
exposes it. Findings should snowball through the graph: a risky package version
or suspicious script in a transitive dependency should be visible at the root
decision point.

The dependency graph should be reusable across CLI output, JSON output, registry
records, and test fixtures.

### 5. Policy Engine

The policy engine turns evidence into a decision.

Its core decisions should be:

- `allow`: proceed without interaction.
- `ask`: show evidence and require approval.
- `deny`: stop execution.
- `override-required`: allow only with an explicit, logged override.

Policy should support both local developer workflows and organization-managed
rules. Examples include denying packages published in the last hour, asking when
lifecycle scripts are present, denying known typo-squats, or requiring an audit
record for agent-driven execution.

### 6. Execution Delegator

The execution delegator runs the original command only after resolution,
verification, evidence extraction, graph construction, and policy evaluation
have completed.

This component should stay small. The safest design is to make everything before
execution inspectable and everything after approval clearly delegated.

### 7. Audit Registry

The registry should store audit records for exact package versions, not broad
package names.

An audit record should include:

- Package name, version, registry, and tarball identity.
- Integrity and artifact digests.
- Evidence extraction timestamp and tool version.
- Lifecycle scripts, binaries, dependency graph summary, and risk findings.
- Links to inherited dependency findings.
- Review status, reviewer identity where applicable, and expiration metadata.

The registry should make repeated decisions cheaper and more consistent. If a
package version has already been inspected, future users and agents should be
able to reuse the evidence instead of starting from zero.

The registry should be optional for local use. The CLI must remain useful without
a hosted service.

### 8. Agent Protocol

`safe-npx` should provide a stable agent-facing contract.

Agents should not need to scrape terminal text. They should receive structured
evidence, policy status, and next-step requirements. The contract should make it
easy for an agent to say: "I am about to run remote package code. Here is the
artifact, here are the risk signals, and here is the decision I need from you."

Over time, this protocol could be adopted by coding agents, package managers,
IDEs, and CI systems.

## Year-One Product Shape

By the one-year mark, the project should plausibly have:

- A production-quality Rust CLI for npm package execution gating.
- Support for exact root artifact verification and dependency graph evidence.
- A documented JSON schema for agents.
- A local policy file format.
- A public alpha audit registry with a small but real package corpus.
- A reproducible malicious-package fixture suite.
- CI examples for fail-closed package execution.
- Documentation for agent vendors and package manager maintainers.
- A governance model for sponsors and contributors.
- Clear language that it provides risk evidence, not safety guarantees.

The success condition is not that every package becomes safe. The success
condition is that blind remote execution becomes harder to justify and easier to
replace with visible, repeatable evidence.

## Design Principles

- Exact artifacts over package-name reputation.
- Evidence before execution.
- Transparent signals over black-box scoring.
- Local usefulness without a hosted registry.
- Registry acceleration when shared evidence exists.
- Human-readable by default, machine-readable by design.
- Fail-closed modes for agents and CI.
- Reproducible fixtures for every important threat.
- Small, auditable Rust components.
- Clear separation between inspection, policy, and execution.

## Open Questions

- How much dependency graph resolution can be done without reimplementing a full
  package manager?
- Which signals are strong enough to block by default, and which should only
  escalate to `ask`?
- How should registry audit records expire as packages, maintainers, and threat
  intelligence change?
- What should the minimum viable agent protocol include?
- How should private package metadata be protected when organizations use a
  shared registry?
- Which package-manager and agent integrations should come first?

These questions should stay visible. The project should move quickly, but it
should not pretend the trust model is simpler than it is.
