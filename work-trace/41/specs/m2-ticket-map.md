# M2 Ticket Map

## Source Deliverables

M2 from `docs/milestones.md` requires:

- execution-closure design note comparing direct execution, pinned local
  tarball execution, and package-manager delegation
- decision record choosing `direct_extract`, `pinned_delegation`, or
  `inspect_only_alpha`
- no-package-code-ran canary harness
- race matrix for resolution time versus execution time
- bin selection rules and refusal behavior
- bin fixtures for single bin, ambiguous bin, missing bin, scoped package bin,
  and forwarded args
- registry and `.npmrc` precedence tests
- closure fixtures for tag moves, cache poisoning, dependency lifecycle escape,
  generated shims, and selected binaries

## Acceptance Criteria Coverage

| M2 acceptance criterion | Planned issue coverage |
| --- | --- |
| Inspection cannot run package binaries, lifecycle scripts, or dependency scripts. | #9, #43, #44, #48 |
| The selected execution mechanism cannot inspect one version and execute another. | #4, #45, #47, #49, #50 |
| Registry metadata changes between inspection and execution fail closed. | #46, #47, #51 |
| Cache entries, generated shims, selected bins, dependencies, and lifecycle scripts cannot escape the verified closure. | #44, #45, #47, #48, #49 |
| If full dependency closure cannot be proven, execution returns `execution_refused` with reason `unsupported_closure`. | #42, #44, #51 |
| If the executable subset is too narrow for useful alpha execution, the alpha ships as inspect-only rather than weakening the invariant. | #4, #50, #52 |

## Existing Issues Preserved And Refined

- #41: plan execution closure spike ticket map. This is the temporary planning
  issue used to create this map and should close before implementation begins.
- #4: choose execution mechanism and prove byte identity. This is now the final
  M2 decision record, dependent on evidence from the other M2 tickets.
- #9: build no-package-code-ran canary harness. This remains an early security
  harness ticket.
- #10: define bin selection and forwarded-argument fixtures. This remains the
  deterministic bin-selection ticket.

## Created Issues

- #42: define execution closure contracts and reason vocabulary.
- #43: extract root artifact safely for static closure inspection.
- #44: detect lifecycle scripts and dependency declarations as closure blockers.
- #45: model selected bin and generated shim byte identity.
- #46: prove registry and `.npmrc` precedence agreement.
- #47: build resolution-to-execution race matrix fixtures.
- #48: seed closure fixture manifest and golden M2 outcomes.
- #49: prototype direct-extract root-only execution with local fixtures.
- #50: evaluate pinned-delegation feasibility without raw `npx` fallback.
- #51: wire `execution_refused` outputs for unproven closures.
- #52: audit and close execution closure spike.

## Suggested Implementation Order

After #41 closes:

1. #42 Define execution closure contracts and reason vocabulary.
2. #9 Build no-package-code-ran canary harness.
3. #43 Extract root artifact safely for static closure inspection.
4. #48 Seed closure fixture manifest and golden M2 outcomes.
5. #44 Detect lifecycle scripts and dependency declarations as closure blockers.
6. #10 Define bin selection and forwarded-argument fixtures.
7. #45 Model selected bin and generated shim byte identity.
8. #46 Prove registry and `.npmrc` precedence agreement.
9. #47 Build resolution-to-execution race matrix fixtures.
10. #51 Wire `execution_refused` outputs for unproven closures.
11. #49 Prototype direct-extract root-only execution with local fixtures.
12. #50 Evaluate pinned-delegation feasibility without raw `npx` fallback.
13. #4 Choose execution mechanism and prove byte identity.
14. #52 Audit and close execution closure spike.

## Dependency Notes

- #42 is the contract foundation and should land before implementation tickets.
- #9 should land early because every later ticket benefits from the same
  no-package-code-ran sentinels.
- #43 enables static package evidence extraction for #44, #10, #45, and #49.
- #48 can start after #42 and #9, then grow as later M2 tickets add fixture
  kinds.
- #4 should not close until the evidence tickets are complete; M5 execute-mode
  planning depends on its outcome.
- #50 is intentionally P1 rather than P0 because pinned delegation may be
  rejected quickly if it cannot preserve byte identity, but the analysis is
  still useful for the #4 decision record.

## Out Of Scope For M2

- Full dependency closure execution beyond the spike proof.
- Broad private registry support.
- Hosted audit registry records.
- M3 full inspect JSON schema.
- M4 policy scoring.
- M5 approval cache and public alpha shipping.
