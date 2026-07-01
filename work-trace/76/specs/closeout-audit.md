# M4 Closeout Audit

Date: 2026-07-01

## Scope

M4 parent: #6, "M4: Implement provisional policy v0 and exit semantics".

Closeout issue: #76, "M4: Close out policy v0 milestone".

GitHub milestone: #5, "M4: Provisional Policy V0 And Exit Semantics".

## Issue State Before Closeout Commit

Closed implementation children:

- #66: policy evaluation model.
- #67: provisional policy thresholds.
- #68: interactive ask semantics.
- #69: non-interactive fail-closed semantics.
- #70: exit code contract.
- #71: required-next-action semantics.
- #72: policy fixture corpus and golden outputs.
- #73: heuristic signals remain report-only.
- #74: decision enum integration coverage.
- #75: policy matrix and exit behavior documentation.

Open before closeout:

- #76: closeout audit, in progress.
- #6: M4 parent, kept open until #76 and closeout CI are complete.

No M4 child implementation issue is deferred.

## Milestone State Before Closeout

GitHub milestone #5 reported:

- state: open
- open issues: 2
- closed issues: 10

The two open issues are #6 and #76. The milestone should be closed only after
#76 is closed and #6 has been closed or updated as complete.

## Documentation Audit

M4 documentation now matches implementation reality:

- `docs/m4-policy-v0.md` records decisions, reasons, next actions, exit codes,
  provisional thresholds, examples, fixture links, and non-goals.
- `docs/roadmap.md` links the M4 policy reference from the M4 roadmap section.
- `README.md` links the M4 policy reference in the repository map.
- `docs/milestones.md` marks M4 as implemented with closeout tracked in #76.

## Verification

Latest green main CI before this closeout evidence:

- commit: `ecd2def059992b01f975bf7d0af2c44d728152c7`
- run: <https://github.com/wonop-io/safenpx/actions/runs/28524344930>
- result: success

Local verification for closeout evidence:

- `just test`: passed locally on 2026-07-01 after the closeout evidence update.

Closeout commit and GitHub Actions run are recorded in the #76 closeout comment
after push, because the run id is only known after the commit reaches `main`.

## Residual Risks

- M4 policy is provisional; thresholds are conservative constants, not final
  security science.
- `safe-npx` still does not prove packages are safe.
- Dependency closure is not verified in M4.
- Execution remains constrained by earlier M2 direct-extract decisions and M5
  execute-alpha work.
- Hosted audit registry, attestations, release diff evidence, and approval
  cache semantics remain future milestones.
