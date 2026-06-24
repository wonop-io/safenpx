# Issue 35 Requirements Brief

## Facts

- M1 supports `name@version` and `@scope/name@version` package specs.
- M1 must not execute package code while resolving metadata.
- Tests must not depend on the live npm registry.
- Existing contracts include `PackageSpec`, `ResolvedPackage`,
  `RegistrySource`, and `M1Reason`.
- Issue #8 seeded registry failure fixtures for `registry_error`,
  `missing_package`, and `missing_version`.

## Requirements

- Add a registry client boundary that accepts a supported `PackageSpec`.
- Build public npm metadata URLs correctly for unscoped and scoped packages.
- Parse registry metadata for the requested exact version.
- Extract the tarball URL and integrity value from the version dist metadata.
- Preserve stable error reasons for missing packages, missing versions,
  invalid registry payloads, and network/transport failures.
- Cover all behavior through stubbed registry responses.

## Non-Goals

- Download tarball bytes.
- Verify integrity.
- Resolve dist-tags such as `latest`.
- Read `.npmrc` or private registry configuration.
- Execute, install, or delegate to npm.

