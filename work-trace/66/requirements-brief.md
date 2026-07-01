# Requirements Brief

## Problem

M4 needs a single auditable policy entrypoint. Current decision semantics are
split across M1 report construction, inspect rendering, JSON schema mapping,
M2 refusal helpers, and exit-code helpers.

## Scope

- Add a policy module with a versioned policy evaluation result.
- Represent decision, reasons, required next action, policy version, and rule
  identifiers together.
- Cover M1 inspect/report evidence and M2 closure refusal reasons.
- Wire inspect and M2 render paths to consume the policy result.

## Acceptance Criteria

- One policy entrypoint evaluates inspect evidence into a decision result.
- The result includes decision, reasons, required next action, policy version,
  and rule identifiers.
- Renderers consume the policy result rather than recomputing decisions.
- Unsupported closure maps to `execution_refused`, not `deny`.
- Integrity mismatch and resolver ambiguity map to `deny`.
- Existing M3 JSON and human output tests remain deterministic.
