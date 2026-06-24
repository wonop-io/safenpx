# M1 Data Contracts

## Acceptance Criteria

- Types represent supported exact-version specs and forwarded args.
- Types represent unsupported/malformed specs without requiring network calls.
- M1 reasons include at least `unsupported_spec`, `malformed_spec`,
  `registry_error`, and `integrity_mismatch`.
- Unit tests cover serialization or debug rendering where used by reports.

## Planned Types

- `CommandIntent`
- `PackageSpec`
- `PackageSpecParse`
- `UnsupportedSpec`
- `ResolvedPackage`
- `RegistrySource`
- `ArtifactIdentity`
- `M1Reason`

