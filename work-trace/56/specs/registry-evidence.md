# Registry Evidence

## Premise

The registry response is useful evidence about what the registry claims for a
resolved exact version. It is not proof of what the tarball contains and must
remain separate from static extraction facts read from verified bytes.

## Evidence Shape

Registry evidence should include:

- selected registry URL or source identity
- package scope category: scoped or unscoped
- resolved package name and version
- publish time when the registry response provides it
- maintainers, publisher, repository, license, and provenance-like fields when
  present and well-formed enough to represent
- dist integrity and tarball URL for the resolved exact version

Optional fields should be absent or empty when missing or malformed. Missing
optional metadata must not fail inspection.

## Fixture Coverage

Tests should cover:

- public npm-style metadata with optional fields populated
- metadata where optional fields are absent
- scoped package metadata
- malformed optional metadata that should be ignored or degraded without
  failing the inspect report

## Boundaries

- Registry evidence is a `registry_evidence` or equivalent fact block, not a
  tarball/package metadata block.
- Static extraction remains tied to verified tarball bytes.
- #12 will refine final authority-context redaction semantics.
