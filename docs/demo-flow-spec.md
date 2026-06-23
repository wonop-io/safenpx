# Demo Flow Spec

Do not implement this demo yet. This document specifies the target flow for the
first public demo.

## Command

```bash
safe-npx create-example@latest
```

## Demo Goal

Show that `safe-npx` gives useful evidence before remote package code runs.

The demo should feel like the missing `npx` prompt:

- exact package identity
- integrity
- publisher and release age
- dependency graph summary
- lifecycle scripts
- package behavior signals
- risk recommendation
- ask-before-execution policy

## Target Human Output

```text
agent@repo:~/workspace
pre-exec
$ safe-npx create-example@latest

Package: create-example@2.4.1
Registry: https://registry.npmjs.org
Integrity: sha512-...
Published: 3 hours ago
Publisher: example-maintainer
Dependencies: 87 resolved nodes
Lifecycle scripts: postinstall
Bin: ./dist/cli.js
Package size: 1.8 MB
Readability: bundled, partially minified
Audit: no verified third-party audit found
Policy: ask before execution

Recommendation: elevated risk

Reasons:
- Recently published version with executable bin
- postinstall script present
- No verified audit for this exact tarball
- 2 dependencies have inherited warnings

Continue? [y/N]
```

## Target JSON Output

```bash
safe-npx --json create-example@latest
```

```json
{
  "schema_version": "0.1",
  "subject": {
    "ecosystem": "npm",
    "registry": "https://registry.npmjs.org",
    "name": "create-example",
    "version": "2.4.1",
    "integrity": "sha512-..."
  },
  "recommendation": "ask",
  "risk": {
    "level": "elevated",
    "score": 72,
    "confidence": "medium"
  },
  "policy": {
    "decision": "allow_with_prompt",
    "matched_rules": []
  },
  "graph": {
    "dependency_count": 87,
    "direct_dependency_count": 12,
    "inherited_finding_count": 2
  },
  "findings": [
    {
      "id": "lifecycle-postinstall",
      "severity": "medium",
      "category": "lifecycle_script",
      "confidence": "high",
      "evidence": "package.json contains postinstall"
    }
  ]
}
```

## Acceptance Criteria

- No package lifecycle script runs before the report is printed.
- No package bin executes before the user or policy allows it.
- The package version and integrity are exact, not inferred from a loose range.
- The output is useful to a human reading the terminal.
- The JSON output is deterministic enough for a coding agent to consume.

