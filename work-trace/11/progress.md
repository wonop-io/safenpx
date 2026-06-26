# Issue 11 Progress

## 2026-06-26

- Moved issue #11 to `status:in-progress`.
- Created trace scaffold before implementation.
- Added root artifact size and regular file count from verified tarball bytes.
- Added optional package evidence parsing for repository, license,
  maintainer-like people, and provenance-like fields without failing on
  malformed optional metadata.
- Marked dependency declarations as declaration-only evidence.
- Added fixture coverage for normal metadata, missing optional metadata,
  lifecycle scripts, dependency declarations, multiple bins, and malformed
  package metadata.
