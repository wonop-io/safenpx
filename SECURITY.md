# Security Policy

`safe-npx` is evidence-before-execution infrastructure. It does not prove that
packages are safe; it aims to make package execution decisions less blind.

## Reporting A Vulnerability

Please do not open a public issue for sensitive vulnerabilities.

Use GitHub Security Advisories:
https://github.com/wonop-io/safenpx/security/advisories/new

If that route is unavailable, contact a repository maintainer through an
out-of-band channel and share only enough information to coordinate a private
report.

## In Scope

- Ways package code can run before inspection or policy evaluation.
- Integrity, cache, registry, or metadata confusion that can make inspected
  bytes differ from executed bytes.
- Fail-open behavior that falls back to raw `npx` or `npm exec`.
- Secret exposure in fixtures, logs, reports, or decision receipts.

## Out Of Scope

- Claims that a third-party npm package is generally safe or unsafe.
- Malware analysis requests for packages that do not affect `safe-npx`
  behavior.
- Social engineering or attacks against maintainers, sponsors, or users.

