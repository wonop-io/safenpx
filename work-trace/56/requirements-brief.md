# Requirements Brief

Issue #56 acceptance criteria:

- Registry evidence records selected registry source and package scope category.
- Publish time is represented when available and absent when missing without
  failing inspection.
- Maintainers, publisher, repository, license, and provenance-like fields are
  represented when available.
- Dist integrity and tarball URL are tied to the resolved exact version.
- Registry metadata is not treated as proof of tarball package contents.
- Fixtures cover public npm metadata, missing optional fields, scoped registry
  metadata, and malformed optional metadata.

Dependencies:

- #54 is closed and provides the explicit inspect pipeline.
- #12 owns final authority-context redaction rules, so this issue should avoid
  printing secrets and keep any deeper redaction semantics minimal.

Non-goals:

- Hosted registry audit service.
- Release diffing or external attestations.
- Treating registry metadata as verified tarball content.
