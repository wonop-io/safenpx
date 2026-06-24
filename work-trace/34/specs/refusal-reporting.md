# Refusal Reporting Spec

## Unsupported Specs

Unsupported specs must report:

- `state: unsupported`
- `reason: unsupported_spec`
- a stable category
- the raw rejected package spec string
- `downloaded: false`

## Malformed Specs

Malformed specs must report:

- `state: malformed`
- `reason: malformed_spec`
- the raw rejected package spec string
- `downloaded: false`

## Supported Specs

Forwarded args are preserved only when the package spec is supported.

## Non-Goals

- No raw `npx` or `npm exec` fallback.
- No package-code execution.
- No registry or tarball access.
