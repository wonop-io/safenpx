# Root Tarball Download

## Behavior

- Accept a `ResolvedPackage` produced by the metadata client.
- Validate that `tarball_url` is absolute `https://` or `http://`.
- Fetch the tarball URL through a `TarballTransport` interface.
- Return an `ArtifactBytes` object containing:
  - package name.
  - package version.
  - source tarball URL.
  - unmodified downloaded bytes.
- Do not extract the tarball.
- Do not run package-manager commands.
- Do not run package binaries, lifecycle scripts, or dependency scripts.

## Error Mapping

- Invalid tarball URL maps to `M1Reason::RegistryError`.
- Transport failures map to `M1Reason::RegistryError`.
- HTTP 404 maps to `M1Reason::RegistryError`.
- Non-success HTTP responses map to `M1Reason::RegistryError`.
- Empty responses map to `M1Reason::RegistryError`.

## Test Requirements

- Tests use local stub transports only.
- Tests cover valid downloads.
- Tests cover invalid URL, transport failure, HTTP failure, and empty body.
- Tests prove the downloader records zero package-manager, extraction, binary,
  lifecycle, and dependency-script attempts.

