# Policy Documentation Plan

## New Durable Document

Create a policy v0 reference under `docs/` that covers:

- Decision vocabulary.
- Reason vocabulary.
- Required next actions.
- Exit-code table.
- Provisional thresholds.
- Representative JSON fixture links.
- Non-interactive ask-required behavior.
- Explicit non-goals and safety limits.

## Link Surface

Link the document from the roadmap or milestone narrative so it is discoverable
from the planning docs.

## Verification

Run `just test` after docs are updated so policy, file-size, and link-adjacent
checks remain green.
