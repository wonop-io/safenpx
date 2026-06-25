# Static Root Extraction Spec

## Input

The extractor accepts tarball bytes plus an `ArtifactIdentity` that was already
verified by M1. Extraction must not resolve the registry again.

## Workspace

The extractor writes to a caller-provided inspection root. Every archive path is
normalized as a relative package path before writing. The implementation rejects:

- absolute paths
- `.` and `..` components
- Windows drive prefixes and path prefixes
- empty paths
- symlinks and hardlinks whose targets are absolute or contain `..`

## Metadata

The extractor reads `package/package.json` or `package.json` from the extracted
workspace and returns parsed package metadata with the original artifact
identity attached.

The first metadata shape only needs the fields later M2 tickets consume:

- package name
- package version
- `bin`
- lifecycle scripts
- dependency declarations

## Execution Boundary

The extractor must only parse archive bytes and JSON. It must not invoke npm,
node, shell commands, lifecycle scripts, package binaries, or generated shims.
