# Requirements Brief

Issue #55 acceptance criteria:

- Inspect-mode tests prove root package binaries do not run during inspection.
- Inspect-mode tests prove lifecycle scripts do not run during inspection.
- Inspect-mode tests prove dependency scripts do not run during inspection.
- Inspect-mode tests prove generated shims do not run during inspection.
- Canary failures are actionable and identify which no-execution invariant
  failed.
- Tests cover both human output and JSON output paths where rendering could
  accidentally trigger evidence collection twice.

Dependencies:

- #54 is closed and provides the explicit `safe-npx inspect <spec>` pipeline.
- #9 is closed and provides the reusable no-package-code canary harness.

Non-goals:

- Add execute mode.
- Contact npm or any external service.
- Build final M3 JSON schema fields or policy decisions.
