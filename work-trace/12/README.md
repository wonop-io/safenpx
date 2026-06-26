# Issue 12 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/12

## Request

Define authority-context redaction rules for M3 inspect reports so the report
can describe ambient process authority without leaking secrets, private tokens,
or machine-specific path details.

## Scope

- Add authority-context categories for registry source, package scope, command
  intent, cwd trust class, runner context, and actor context.
- Redact display values for human and JSON reports.
- Keep redacted display values separate from canonical identity fields reserved
  for receipts and cache keys.
- Include examples for local terminal, CI, trusted project directory, temp
  directory, public npm, scoped registry, manual user, and coding agent.
- State clearly that authority context is not sandboxing.

## First Commit Rule

This trace scaffold is committed before implementation begins.
