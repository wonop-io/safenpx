#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/clean-repo-before-tests/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

if [ "${REQUIRE_CLEAN_REPO:-0}" != "1" ]; then
  echo "Clean worktree gate is available; set REQUIRE_CLEAN_REPO=1 to enforce it."
  exit 0
fi

if [ -n "$(git status --porcelain=v1 --untracked-files=all)" ]; then
  echo "Worktree must be clean before this test gate." >&2
  print_policy_guide_notice
  git status --short >&2
  exit 1
fi

echo "Worktree is clean."
