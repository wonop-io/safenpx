#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/migrations-immutable/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
[ -n "${repo_root}" ] || exit 0
cd "${repo_root}"
. policies/_lib/common.sh

base_ref="$(resolve_policy_base_ref "${1:-}")"
violations=""
checked_count=0

while IFS= read -r path; do
  case "${path}" in
    *migrations/*.sql|*migration*.sql) ;;
    *) continue ;;
  esac
  checked_count=$((checked_count + 1))
  if git cat-file -e "${base_ref}:${path}" 2>/dev/null; then
    violations="${violations}${path}: existing migrations are immutable; create a new migration"$'\n'
  fi
done < <(git diff --name-only --diff-filter=M "${base_ref}" HEAD)

if [ -n "${violations}" ]; then
  echo "Existing migrations must not be edited." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Checked ${checked_count} modified migration path(s); existing migrations are unchanged."
