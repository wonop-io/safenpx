# Issue 57 Work Trace

Issue: https://github.com/wonop-io/safenpx/issues/57

## Request

Model M3 inspect facts, heuristics, decisions, next actions, authority context,
and execution state separately so users and agents do not confuse warnings,
declarations, and verified facts.

## Scope

- Add serialization-ready Rust structures shared by human and JSON renderers.
- Keep package and registry facts separate from heuristic signals.
- Keep heuristics report-only in M3.
- Preserve dependency declarations as declarations unless a later verified
  closure explicitly proves dependency artifacts.
- Include an initial authority-context shape without implementing #12
  redaction rules or #60 source-context depth.

## First Commit Rule

This trace scaffold is committed before implementation begins.
