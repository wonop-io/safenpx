# Threat Model

`safe-npx` is a pre-execution evidence gate. It does not prove that packages are
safe. It reduces blind execution by surfacing risk signals before `npx` or
`npm exec` runs remote package code.

## Protected Decision

The protected decision is:

> Should this exact package artifact be allowed to execute in this environment?

The answer may be `allow`, `ask`, or `deny`, and must include evidence.

## In Scope

### Malicious Packages

Packages intentionally designed to steal secrets, modify local files, download
remote payloads, install persistence, run shell commands, or exfiltrate data.

Relevant signals:

- lifecycle scripts
- executable bins
- suspicious filesystem access
- suspicious network access
- shell execution
- remote code download patterns
- bundled or obfuscated code
- native binaries or compiled addons

### Compromised Maintainers

Legitimate packages can become risky after maintainer account compromise,
token theft, package transfer, or malicious release automation.

Relevant signals:

- newly published version
- changed publisher or maintainer
- unusual package size or file manifest delta
- new lifecycle scripts
- new executable bins
- changed repository metadata
- missing verified audit record for the exact artifact

### Typo-Squats And Name Squats

Attackers can publish names that are visually or semantically close to popular
packages, then rely on humans or agents making small mistakes.

Relevant signals:

- edit-distance similarity to popular package names
- homoglyph or lookalike characters
- confusing scope/name pairs
- low adoption with high similarity to a known project
- disputed or recently transferred package names

### Lifecycle Scripts

Install lifecycle scripts can execute before the user believes they are running
the actual tool.

Relevant signals:

- `preinstall`
- `install`
- `postinstall`
- shell scripts
- binary downloads
- environment variable access

### Agent-Driven Blind Execution

Coding agents may run commands from documentation, generated plans, tool
instructions, or `SKILL.md`-style workflows without human inspection.

Relevant controls:

- JSON output
- deterministic policy decision
- exact artifact identity
- human escalation message
- no execution until policy allows it

## Out Of Scope For v0.1

- Proving package safety.
- Full JavaScript semantic analysis.
- Runtime sandboxing.
- Replacing NPM resolution.
- Blocking all malicious packages.
- Enterprise policy sync.
- Private registry support beyond basic configuration discovery.

## Trust Boundaries

- The NPM registry provides metadata and tarball URLs.
- The local package manager may be used for resolution, with scripts disabled.
- The local filesystem contains sensitive files that package code must not reach
  before the gate completes.
- Optional future registry uploads must be opt-in and must not leak private
  package names or private registry metadata by default.

## Security Posture

`safe-npx` should fail closed when it cannot identify the exact artifact or
cannot determine whether lifecycle scripts would run. It should use language
such as "unknown risk" rather than "safe".

