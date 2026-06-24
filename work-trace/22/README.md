# Work Trace: Issue 22

Issue: `#22` M0: Document issue-backed contributor and agent workflow

## Request

Move the Wonop `plan-work` skill into this repository as a repository playbook
named `plan-issue`.

## Scope

- Convert the skill concept into a playbook instead of a Codex skill.
- Plan one small GitHub issue at a time.
- Store planning artifacts under `work-trace/{issue-id}/...`.
- Require the work-trace scaffold to be checked in before implementation work
  begins.

## Initial Context

- Source skill: `/Users/tfr/Documents/Projects/wonop/.wonopcode/skills/plan-work`.
- Source playbook convention:
  `/Users/tfr/Documents/Projects/wonop/playbooks/repository/playbook-specification.md`.
- Target repository issue: `#22`.

## Progress

- [x] Checked that issue `#22` exists.
- [x] Created checked-in work-trace scaffold for this issue.
- [x] Add the `plan-issue` playbook.
- [x] Add playbook navigation.
- [x] Adopt playbook-spec policy validation.
- [x] Run `just test`.
