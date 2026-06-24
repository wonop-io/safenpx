# Contributing

`safe-npx` is early public infrastructure. The project is deliberately narrow:
collect evidence before package execution, fail closed when the execution
closure cannot be proven, and avoid presenting risk signals as safety proof.

## Pick Up Work

- Start from GitHub Issues. Non-trivial work should already have an issue, or
  you should open one before starting.
- Use the public roadmap project for status:
  https://github.com/orgs/wonop-io/projects/6
- Prefer small issues with clear acceptance criteria.
- For deeper planning, use `playbooks/repository/plan-issue.md` and store
  artifacts under `work-trace/{issue-id}/`.

## Before Opening A Pull Request

- Link the issue with `Refs #123` or `Closes #123`.
- Use conventional commits such as `docs:`, `feat:`, `fix:`, `test:`, or
  `chore:`.
- Run:

```bash
just test
```

## Pull Request Notes

Every PR should explain:

- The linked issue.
- What changed.
- How it was verified.
- Security impact.
- Documentation impact.

