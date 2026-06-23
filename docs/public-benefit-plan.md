# Public Benefit Plan

`safe-npx` should be useful to the open-source ecosystem even if no hosted
commercial service ever appears.

## Permissive OSS License

The project is intended to use the Apache License, Version 2.0.

Why:

- Package-manager maintainers can reuse ideas or code.
- Agent vendors can integrate without licensing friction.
- Security researchers can build fixtures and tooling around it.
- Companies can adopt local policy without a hosted dependency.
- Contributors and sponsors get explicit patent-license hygiene for shared
  security infrastructure.

## Reproducible Test Corpus

The project should maintain a reproducible package test corpus covering:

- benign executable package
- package with `postinstall`
- package with `preinstall`
- package with executable bins
- typo-squat lookalike package
- bundled/minified package
- package with native binary
- package with suspicious network access
- package with suspicious filesystem access
- compromised-maintainer simulation fixture

Every fixture should include:

- package source
- generated tarball
- expected findings
- expected JSON output
- reason the fixture exists

## Documentation For Package Managers

The project should document:

- exact artifact identity model
- resolver assumptions
- integrity handling
- lifecycle-script behavior
- what package-manager APIs or commands are needed
- how to reproduce findings
- where `safe-npx` deliberately delegates rather than reimplements resolution

## Documentation For Agent Vendors

The project should document:

- JSON schema
- policy decision meanings
- recommended agent behavior for `allow`, `ask`, and `deny`
- how to present evidence to a user
- when to stop execution
- how to cache or attach audit records without leaking private package names

## Public Registry Principles

If a package evidence registry is built later:

- uploads are opt-in
- private package metadata is never uploaded by default
- exact artifact integrity is the audit key
- third-party audit records are distinguishable from local heuristic reports
- stale findings are versioned rather than silently overwritten
- self-hosting guidance is published
