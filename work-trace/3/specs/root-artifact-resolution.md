# Root Artifact Resolution

## Behavior

- Accept a supported exact-version `PackageSpec`.
- Resolve public npm metadata through `NpmMetadataClient`.
- Download the resolved root tarball bytes through `TarballDownloader`.
- Verify downloaded bytes against resolved npm integrity metadata.
- Return a `VerifiedRootArtifact` containing:
  - `ResolvedPackage`.
  - `ArtifactIdentity`.
- Do not invoke package managers.
- Do not extract archives.
- Do not run package binaries, lifecycle scripts, or dependency scripts.

## Failure Mapping

- Missing package maps to `M1Reason::MissingPackage`.
- Missing version maps to `M1Reason::MissingVersion`.
- Registry, metadata, transport, HTTP, invalid URL, and empty download failures
  map to `M1Reason::RegistryError`.
- Integrity mismatch, missing integrity, malformed integrity, and unsupported
  integrity algorithms map to `Decision::Deny` /
  `M1Reason::IntegrityMismatch`.

## Test Requirements

- Tests use stubbed metadata and tarball transports only.
- Tests cover successful root artifact resolution.
- Tests cover missing package, missing version, registry failure, download
  failure, and integrity mismatch.
- Tests prove zero package-manager, extraction, binary, lifecycle, and
  dependency-script attempts.

