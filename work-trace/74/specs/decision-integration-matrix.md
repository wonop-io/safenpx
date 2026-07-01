# Decision Integration Matrix

## Objective

Add CLI-level tests that bind policy decision, policy reasons, required next
action, renderer agreement, and process exit behavior together.

## Planned Decision Coverage

| Decision | Expected route | Primary assertions |
| --- | --- | --- |
| `allow` | clean inspect fixture | exit 0, decision allow, next action none, no policy reasons |
| `ask` | lifecycle-script inspect fixture | ask exit, decision ask, lifecycle reason, ask-before-execution next action |
| `deny` | static-blocker inspect fixture | deny exit, decision deny, blocker reason, do-not-execute next action |
| `unsupported` | unsupported spec fixture | unsupported exit, unsupported reason, no execution |
| `inspection_error` | integrity or tarball failure fixture | inspection-error exit, error reason, no execution |
| `execution_refused` | non-interactive ask execution path | refused exit, ask reason, ask-before-execution next action |

## Renderer Agreement

Where a decision supports both output modes, the tests should compare human and
JSON execution of equivalent fixture inputs. JSON remains the canonical source
for exact enum and reason values; human output must show the matching decision,
next action, and relevant reason text.

## Execution Safety

Fixtures must continue using inert tarballs and canary assertions. CLI-level
tests may simulate execution refusal, but must not run package lifecycle code.
