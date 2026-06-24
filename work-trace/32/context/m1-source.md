# M1 Source Context

M1 goal:

Turn a supported `safe-npx` command into exact package coordinates and a
verified root artifact without running package code.

Supported v0 inputs:

- `name@version`
- `@scope/name@version`
- forwarded package arguments after `--`

Unsupported in v0:

- unversioned names
- version ranges other than `latest`
- Git URLs
- local paths
- tarball URLs
- aliases
- multiple package specs
- `npm exec --package`, `-c`, and package-manager-specific variants

M1 accepts `latest` only after tag-race proof, so exact versions are the first
implementation path.

