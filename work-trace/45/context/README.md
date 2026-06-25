# Context

## Facts

- M2 requires execution closure evidence before any package code can run.
- Issue #45 depends on execution closure contracts, static root extraction, and
  deterministic bin selection from #42, #43, and #10.
- Selected bin identity must be derived from verified extracted bytes, not from
  npm-generated shims or package-manager behavior.

## Assumptions

- M2 can refuse generated shims unless their deterministic byte identity is
  explicitly modeled.
- All tests must use local fixtures and must not execute third-party package
  code.

