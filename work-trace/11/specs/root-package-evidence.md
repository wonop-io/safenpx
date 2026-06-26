# Root Package Evidence

## Premise

Root package evidence is read from verified tarball bytes and the extracted
`package.json`. It is stronger than registry metadata for package contents, but
dependency entries remain declarations until a later verified closure proves
the referenced bytes.

## Evidence Shape

Root package evidence should include:

- package byte size computed from the verified tarball bytes
- package file count computed while inspecting the tarball
- selected bin candidates from `package.json`
- forwarded args from the requested command
- lifecycle script facts from `package.json`
- dependency declarations grouped by runtime, optional, peer, bundled, and dev
- repository, license, maintainers-like fields, and provenance-like package
  fields when present and well-formed enough to represent

Optional fields should be absent or empty when missing or malformed. Missing
optional metadata must not fail inspection.

## Boundaries

- Package facts come from verified root artifact bytes.
- Dependency declarations are not verified closure evidence.
- Static extraction must not run package binaries, lifecycle scripts,
  dependency scripts, or install hooks.
- Human/JSON consumers must be able to distinguish facts from warnings and
  decisions, but the final shared evidence model is owned by #57.

## Fixture Coverage

Tests should cover:

- normal package metadata
- missing optional metadata
- lifecycle scripts
- dependency declarations across all supported groups
- multiple bins
- malformed package metadata failure without execution
