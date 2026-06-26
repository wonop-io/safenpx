# Requirements Brief

## Problem

M3 inspect mode needs provisional latency budgets so the POC feels usable and
future regressions are visible. The budgets must be measurable in CI without
depending on live public npm network timing.

## Acceptance Mapping

- Fixture-backed local latency measurement exists.
- Optional live public-npm measurement is documented separately.
- Cold and warm budget targets are recorded.
- CI avoids live-network timing gates.
- Measurements expose resolve, download, extract, and render phases.

## Constraints

- Deterministic tests should avoid wall-clock assertions where possible.
- Any wall-clock measurement must use local fixture work and conservative
  thresholds.
- Do not claim public npm latency budgets are proven by CI fixture tests.
