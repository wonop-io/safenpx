# Context

- Source milestone: `docs/milestones.md` M2.
- Issue #50 feeds the final M2 execution-mechanism decision in #4.
- Package-manager delegation must not re-resolve, run lifecycle hooks, generate
  untracked shims, mutate cache, or execute bytes outside the verified closure.

