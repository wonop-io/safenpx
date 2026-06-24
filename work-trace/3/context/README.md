# Issue 3 Context

## Source Inputs

- GitHub issue #3.
- `docs/milestones.md` M1 root artifact acceptance criteria.
- Issue #35 metadata client.
- Issue #36 tarball downloader.
- Issue #37 integrity verifier.

## Assumptions

- The first integration API can operate on `PackageSpec`; CLI/report wiring can
  call it in issue #39.
- Download failures continue to map to `M1Reason::RegistryError` until a more
  specific artifact-download reason exists in the shared vocabulary.
- The integration layer should not extract tarballs; extraction belongs to later
  evidence or execution-closure milestones.

