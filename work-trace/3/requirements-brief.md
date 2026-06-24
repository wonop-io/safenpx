# Issue 3 Requirements Brief

## Facts

- Issue #35 added public npm metadata resolution behind a stub-friendly
  transport.
- Issue #36 added byte-only root tarball download behind a stub-friendly
  transport.
- Issue #37 added SHA-512 npm integrity verification and stable
  `ArtifactIdentity`.
- Parser-level unsupported and malformed specs already stop before network work.

## Requirements

- Add an end-to-end root artifact resolver for supported exact-version specs.
- The success result must include package name, version, registry URL, tarball
  URL, integrity metadata, and digest identity.
- Missing packages, missing versions, registry failures, download failures, and
  integrity mismatches must produce stable M1 reasons.
- Integrity mismatch must return `Decision::Deny` and
  `M1Reason::IntegrityMismatch`.
- Tests must use local stub transports and prove no package binaries,
  lifecycle scripts, dependency scripts, archive extraction, or package-manager
  commands run.

## Non-Goals

- CLI report wiring; issue #39 owns human and JSON report integration.
- Dist-tags and `latest`.
- Dependency closure verification.
- Tarball extraction or package execution.

