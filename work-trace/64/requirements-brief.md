# Requirements Brief

## Problem

M3 moves safe-npx from repository scaffold into an inspect-first POC. The docs
need to teach users and sponsors what can be tried now, what the evidence means,
and where the safety boundary still is.

## Acceptance Mapping

- README links to the inspect-first workflow and JSON output.
- Docs state that M3 does not broadly execute package code.
- Docs state that evidence signals are decision support, not proof of safety.
- Supported and unsupported M3 command shapes are listed.
- Redaction and authority-context behavior are documented.
- Schema compatibility and reserved fields are documented.

## Constraints

- Keep execute-mode claims constrained to later milestones.
- Do not imply that reserved evidence fields are populated in M3.
- Keep examples deterministic and aligned with existing fixtures.
