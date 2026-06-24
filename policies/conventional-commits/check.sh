#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/conventional-commits/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
[ -n "${repo_root}" ] || exit 0
cd "${repo_root}"
. policies/_lib/common.sh

base_ref="$(resolve_policy_base_ref "${1:-}")"
violations=""
checked_count=0

while IFS= read -r subject; do
  [ -n "${subject}" ] || continue
  checked_count=$((checked_count + 1))
  if ! printf '%s\n' "${subject}" | grep -Eq '^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\([A-Za-z0-9_.-]+\))?!?: .+'; then
    violations="${violations}${subject}"$'\n'
  fi
done < <(git log --format=%s "${base_ref}..HEAD")

if [ -n "${violations}" ]; then
  echo "Commit subjects must use Conventional Commits." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Checked ${checked_count} commit subject(s); Conventional Commits are satisfied."
