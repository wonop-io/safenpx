# Issue 63 Work Trace

GitHub issue: https://github.com/wonop-io/safenpx/issues/63

This trace covers provisional M3 inspect latency budgets. The work must make
latency measurable without turning CI into a flaky live-network timing gate.

## Scope

- Add repeatable fixture-backed inspect latency measurement.
- Record cold and warm budget targets.
- Keep optional public npm timing separate from deterministic tests.
- Capture enough phase evidence to distinguish resolve, download, extract, and
  render bottlenecks.
