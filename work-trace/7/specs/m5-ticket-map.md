# M5 Ticket Map

M5 goal: run only verified execution closures, or keep unsupported shapes
inspect-only/refused without weakening the evidence-before-execution invariant.

## Parent

- #7: M5 parent and milestone index.

## Planned Tickets

1. #77: define execute command contract and gating.
2. #78: carry artifact identity from inspection into execution.
3. #79: implement direct-extract root-only process launch.
4. #80: add interactive approval prompt for execute mode.
5. #81: keep JSON and agent flows inspect-only unless execute is explicit.
6. #13: define experimental approval cache and decision receipts.
7. #82: add approval cache authority-context golden tests.
8. #83: refuse `latest` execution until tag-race proof exists.
9. #84: publish execute-alpha compatibility matrix.
10. #85: package alpha distribution for dogfooding.
11. #86: add alpha help and shell-surface documentation.
12. #87: record dogfood runs and bypass-pressure notes.
13. #88: audit and close execute mode alpha milestone.

## Acceptance Criteria Coverage

| M5 acceptance criterion | Planned issue coverage |
| --- | --- |
| Saying no runs nothing from the package. | #80, #79, #87 |
| Saying yes runs only from the verified execution closure. | #78, #79, #80 |
| Approval in one authority context does not silently apply to a materially broader context. | #13, #82 |
| `latest` cannot inspect one version and execute another. | #83, #78 |
| Dogfood runs record latency, unsupported command shapes, policy decisions, bypass pressure, and workaround behavior. | #84, #85, #87 |
| Approval cache starts experimental or disabled until authority-context golden tests pass. | #13, #82 |

## Deliverable Coverage

| M5 deliverable | Planned issue coverage |
| --- | --- |
| Execute mode for exact-version specs that pass closure proof. | #77, #78, #79 |
| Later support for `latest` after tag-race fixtures pass. | #83 |
| Approval prompt for interactive mode. | #80 |
| Explicit inspect-only behavior for JSON unless execute is chosen. | #81 |
| Local approval cache and invalidation. | #13, #82 |
| Compatibility matrix. | #84 |
| Alpha package distribution for dogfooding. | #85 |
| Install, upgrade, and uninstall notes. | #85 |
| Help, shell completion, or manpage coverage. | #86 |
| Milestone closeout evidence. | #88 |

## Suggested Implementation Order

After #7 planning closes:

1. #77 Define the execute contract.
2. #78 Carry artifact identity into launch.
3. #83 Refuse moving specs while preserving exact-version execution.
4. #81 Keep JSON and agent flows inspect-only unless execute is explicit.
5. #80 Add the interactive approval prompt.
6. #79 Implement direct-extract root-only process launch.
7. #82 Add cache authority-context golden tests.
8. #13 Implement experimental approval cache and decision receipts.
9. #84 Publish the compatibility matrix.
10. #86 Add help/reference coverage.
11. #85 Package the alpha distribution path.
12. #87 Record dogfood and bypass-pressure evidence.
13. #88 Audit and close M5.

## Dependency Notes

- #77 should land first because it freezes the alpha CLI contract.
- #78 and #83 are safety foundations before broader launch work.
- #79 depends on #78 and should remain fixture-first until the launch invariant
  is proven by tests.
- #13 should not enable cache behavior before #82 passes.
- #84 can begin early, but should be finalized after #77, #83, and #79 clarify
  supported and refused command shapes.
- #85 should wait until the execute surface and compatibility matrix are stable
  enough for dogfooding.
- #88 closes only after all M5 children are complete or explicitly deferred.

## Out Of Scope For M5

- Full dependency-closure execution.
- Raw package-manager delegation.
- Hosted audit registry records.
- Broad private registry support.
- Release attestations beyond documented alpha release expectations.
- M6 fixture benchmark packaging and decision-quality studies.

