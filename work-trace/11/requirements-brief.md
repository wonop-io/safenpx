# Requirements Brief

Issue #11 acceptance criteria:

- Inspect extraction never runs package binaries, lifecycle scripts, dependency
  scripts, or install hooks.
- Package size and file count are computed from verified artifact bytes.
- Binaries and lifecycle scripts are reported as facts.
- Runtime, optional, peer, bundled, and dev dependency declarations are
  reported as declarations, not verified closure.
- Missing repository, license, provenance, or maintainer-like optional fields
  do not fail inspection.
- Fixture coverage includes normal package metadata, missing optional fields,
  lifecycle scripts, dependency declarations, multiple bins, and malformed
  package metadata.
- Human and JSON consumers can distinguish facts from heuristic warnings and
  decisions.

Dependencies:

- #54 provides the inspect pipeline.
- #55 provides no-package-code canary coverage on the real inspect path.
- #56 provides registry metadata evidence, separate from tarball facts.

Non-goals:

- Verifying dependency closure.
- Executing package binaries or lifecycle hooks.
- Final M3 human report formatting, which remains with #58.
- Final JSON schema compatibility rules, which remain with #5 and #59.
