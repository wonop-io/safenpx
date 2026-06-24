# GitHub Tools

These scripts let Codex and maintainers operate the online planning system from the repo.

## Requirements

- GitHub CLI: `gh`
- Authenticated session with repo access.
- Project operations need the `project` scope.

Run:

```sh
gh auth login
gh auth refresh -s project
```

## Commands

- `just roadmap-bootstrap`: create labels, milestones, initial issues, and a roadmap project where possible.
- `just roadmap-status`: print issue and project state.
- `just issue-list`: list open issues.
- `just issue-view ISSUE=123`: inspect one issue.
- `just issue-start ISSUE=123`: assign the issue to the current GitHub user and mark it in progress by labels.
- `just issue-done ISSUE=123`: close the issue as completed.
