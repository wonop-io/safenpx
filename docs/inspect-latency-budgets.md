# Inspect Latency Budgets

M3 uses provisional latency budgets so inspect mode stays usable and regressions
leave evidence.

## Targets

- Cold public-package inspect: under 5 seconds.
- Warm fixture-backed inspect: under 1 second.

These are product budgets, not safety guarantees. They may change after
dogfooding, fixture expansion, and real public npm measurements.

## CI Measurement

CI validates the latency evidence shape, budget constants, and phase accounting
with deterministic tests. CI does not enforce wall-clock latency for either the
fixture or live public npm profiles because host and network timing would make
the gate flaky.

The fixture-backed profile records these phases:

- `resolve_ms`: registry metadata resolution, JSON parsing, and integrity
  verification outside tarball transport time,
- `download_ms`: tarball byte transport,
- `extract_ms`: static tarball extraction and package metadata parsing,
- `render_ms`: human or JSON report rendering.

The phase split is intentionally coarse but actionable enough to tell whether a
regression is mostly registry resolution, tarball transfer, extraction, or
report rendering.

## Local Fixture Measurement

Run the repeatable no-network fixture measurement with:

```bash
just latency-fixture
```

This runs an ignored Rust test and prints JSON evidence. It is useful before
and after inspect-pipeline changes because it exercises a local package fixture
with a bin, lifecycle script, and dependency declaration.

## Optional Live Measurement

Run a coarse live public npm measurement manually with:

```bash
just latency-live
```

The default package is `is-number@7.0.0`; pass another exact package when the
change under review needs a different shape:

```bash
just latency-live PACKAGE='create-vite@5.5.5'
```

This command measures end-to-end CLI latency, including cargo/process overhead.
It does not emit phase evidence yet. Use `just latency-fixture` when a phase
breakdown is needed, and record live results in the relevant issue or work
trace when they influence a decision.
