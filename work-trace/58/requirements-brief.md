# Requirements Brief

## Problem

The M3 human report must let humans understand inspect evidence without reading
JSON. It must separate facts from heuristics and make the safety boundary plain.

## Acceptance Mapping

- Separate facts, heuristics, decision, reasons, required next action, and
  authority context.
- Include M3 evidence v0 fields when available.
- Label dependency declarations as declarations, not verified closure.
- State that risk signals are not proof that a package is safe.
- State what safe-npx catches and does not catch.
- Keep authority and privacy redaction in the human output.
- Add golden snapshots for normal evidence, lifecycle/dependency blockers,
  unsupported specs, missing optional metadata, and redacted authority context.

## Constraints

- Use the shared inspect evidence model.
- Do not add network-dependent tests.
- Do not expose secrets, raw home paths, or environment dumps.
- Keep file sizes under policy limits.

