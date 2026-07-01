# Heuristics Report-Only

## Intent

M4 policy may ask on documented threshold findings such as lifecycle scripts,
recent publish, large tarball, or large file count. M3 heuristic records remain
report-only unless a later ticket promotes a specific heuristic into a policy
rule with tests, docs, and schema expectations.

## Guardrail Plan

- Verify heuristic records are rendered as report-only.
- Verify heuristic-only unusual-shape evidence does not become `deny`.
- Verify heuristic-only facts can remain `allow` when caller policy is allow.
- Verify JSON exposes heuristic records without treating them as policy reasons.
