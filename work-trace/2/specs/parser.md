# Parser Spec

## Supported

- `name@version`
- `@scope/name@version`
- forwarded args after `--`

## Unsupported

- unversioned names
- `latest` and other floating/range intent in M1
- Git URLs
- local paths
- tarball URLs
- aliases
- multiple package specs
- `npm exec` variants

## Acceptance Criteria

- Supported specs produce parsed `name`, `version`, and `forwarded_args`.
- Malformed specs do not require network calls.
- Unsupported specs return `unsupported_spec`.
- Tests cover scoped packages and forwarded args.

