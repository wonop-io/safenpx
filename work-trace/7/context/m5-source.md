# M5 Planning Source

Date: 2026-07-03

## Inputs

- `docs/milestones.md` M5: Execute Mode Alpha.
- `docs/roadmap.md` M5: Execute Alpha And Adoption.
- `docs/m2-execution-decision.md`: first alpha execution path is
  `direct_extract`.
- Issue #7: M5 parent and milestone index.
- Issue #13: existing approval cache ticket, preserved and refined.
- `just roadmap-status`: M5 had #7 and #13 open before this planning pass.

## M2 Constraint Carried Forward

M5 may prototype execute mode only for the `direct_extract` root-only subset:
exact-version package specs, verified root artifact extracted by `safe-npx`, one
deterministic selected root bin, selected-bin bytes hashed before launch,
forwarded argv preserved, no lifecycle scripts, no dependency declarations
unless full dependency closure is proven, registry-source agreement, and no
package-manager delegation.

## Planning Notes

- #7 is the parent and remains open until M5 closeout.
- #13 remains the approval-cache implementation ticket.
- #77 through #88 were created as M5 child issues.
- #77 was repaired after an initial shell quoting error polluted the issue body.
  The repaired body was verified through `gh issue view`.

