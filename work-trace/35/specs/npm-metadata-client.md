# npm Metadata Client

## Behavior

- Resolve only a previously parsed `PackageSpec`.
- Use `https://registry.npmjs.org/{encoded_name}` for public npm metadata.
- Encode scoped package names as `%40scope%2Fname`.
- Select exactly `versions[package_spec.version]`.
- Return `ResolvedPackage` with:
  - package name from the requested spec.
  - version from the selected version metadata.
  - registry source URL.
  - `dist.tarball`.
  - `dist.integrity`.

## Error Mapping

- HTTP 404 maps to `M1Reason::MissingPackage`.
- Missing requested version maps to `M1Reason::MissingVersion`.
- Transport errors map to `M1Reason::RegistryError`.
- Non-success HTTP responses other than 404 map to
  `M1Reason::RegistryError`.
- Invalid JSON, missing `versions`, missing `dist.tarball`, missing
  `dist.integrity`, or version-name mismatch maps to
  `M1Reason::RegistryError`.

## Test Requirements

- Tests use stubbed registry responses and do not hit live npm.
- Tests cover unscoped URL construction.
- Tests cover scoped URL construction.
- Tests cover successful exact-version metadata extraction.
- Tests cover missing package, missing version, invalid payload, and transport
  failure mapping.

