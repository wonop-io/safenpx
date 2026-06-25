# Context

## Facts

- M2 requires the selected binary to be part of the execution closure.
- Issue #10 depends on the execution closure contracts from #42 and static root
  extraction from #43.
- Issue #45 depends on these bin-selection rules before it can record selected
  bin byte identity.

## Assumptions

- M2 supports exact-version package specs only.
- No fixture or test may execute third-party package code.

