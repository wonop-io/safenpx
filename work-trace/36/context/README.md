# Issue 36 Context

## Source Inputs

- GitHub issue #36.
- `docs/milestones.md` M1 tarball download deliverable.
- Issue #35 npm metadata client and `ResolvedPackage` contract.
- Issue #8 artifact fixture manifest.

## Assumptions

- Download failures should use `M1Reason::RegistryError` until a separate
  artifact-download reason is introduced.
- Empty byte responses are invalid artifact responses and should fail closed.
- In-memory bytes are acceptable for M1; persistent cache semantics can follow
  after integrity verification exists.

