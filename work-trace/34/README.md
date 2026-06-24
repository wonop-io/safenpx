# Issue 34 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/34

## Request

Implement unsupported-spec refusal behavior for M1.

## Scope

- Return stable unsupported or malformed classifications for unsupported inputs.
- Include what was rejected and whether any bytes were downloaded.
- Preserve forwarded args only for supported specs.
- Keep the CLI inspect-only; do not add any fallback to raw `npx` or `npm exec`.

## First Commit Rule

This trace scaffold is committed before implementation begins.
