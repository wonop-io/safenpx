# Issue 36 Requirements Brief

## Facts

- Issue #35 added `ResolvedPackage` metadata with `tarball_url` and `integrity`.
- M1 needs root artifact bytes before integrity verification in issue #37.
- Downloading bytes must not invoke npm, npx, package binaries, lifecycle
  scripts, dependency scripts, archive extraction, or dependency installation.
- Tests must be local and deterministic.

## Requirements

- Add a downloader interface that tests can stub.
- Return downloaded bytes together with the source tarball URL and package
  coordinates.
- Map transport, HTTP, empty-body, and invalid URL failures to stable M1
  errors.
- Add tests proving the download path only requests bytes through the transport
  and records zero execution-like side effects.

## Non-Goals

- Integrity verification.
- Digest identity.
- Tar archive extraction.
- Cache eviction policy.
- Package-manager delegation.

