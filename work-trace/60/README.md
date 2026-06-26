# Issue 60 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/60

## Request

Model caller-declared source context for M3 inspect reports so humans and
agents can distinguish manual, docs, agent, CI, and unknown origins without
pretending safe-npx can infer intent.

## Scope

- Add a source-context enum with manual terminal, docs snippet, agent skill, CI,
  and unknown categories.
- Choose a V0 input path that is caller-declared or defaults to unknown.
- Include source context in the shared inspect model, human output, and JSON
  output.
- Document invalid input behavior.
- Cover every category with deterministic tests.

## First Commit Rule

This trace scaffold is committed before implementation begins.
