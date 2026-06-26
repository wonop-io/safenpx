# Shared Inspect Model

## Premise

M3 inspect output should have one shared model that both human and JSON
renderers consume. The model must make it hard to confuse verified facts,
declaration-only evidence, report-only heuristics, decisions, and execution
state.

## Model Shape

The shared inspect model should include distinct fields for:

- facts: command intent, registry evidence, artifact identity, root package
  extraction evidence, and failure/no-download facts
- heuristics: report-only signals such as lifecycle scripts, dependency
  declarations, similar-name placeholders, or unusual package shape signals
- decision: current recommendation, stable reasons, and required next action
- authority context: initial command/registry/package-scope context, with
  deeper redaction deferred to #12
- execution state: explicit no-execution or failure state for M3 inspect

## Boundaries

- Heuristics are report-only in M3. They may inform future policy, but this
  ticket must not turn them into new hard denials.
- Dependency declarations remain `declaration_only` and must not be serialized
  as verified dependency artifact facts.
- Missing optional package or registry facts should remain absent or empty, not
  inspection failures.

## Fixture Coverage

Tests should cover:

- fact-only evidence
- heuristic evidence
- unsupported or refusal evidence
- missing optional facts
- shared model consumption by both human and JSON rendering paths
