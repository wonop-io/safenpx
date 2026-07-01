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

## Audit Notes

- Current inspect heuristics are lifecycle scripts, dependency declarations, and
  unusual package shape.
- No similar-name heuristic exists in the current codebase; this ticket guards
  the active heuristic surface and records that similar-name must get explicit
  policy tests if introduced later.
- Lifecycle scripts are the only current heuristic-like signal that contributes
  to `ask`, and it does so through the documented M4 lifecycle policy rule.
