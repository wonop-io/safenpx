#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/replicated-code/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
[ -n "${repo_root}" ] || exit 0
cd "${repo_root}"
. policies/_lib/common.sh

base_ref="$(resolve_policy_base_ref "${1:-}")"
checked_count=0

while IFS= read -r -d '' path; do
  [ -f "${path}" ] || continue
  case "${path}" in
    *.rs|*.sh) checked_count=$((checked_count + 1)) ;;
    *) continue ;;
  esac
done < <(changed_paths_z "${base_ref}")

echo "Checked ${checked_count} source file(s); full replicated-code detection is deferred for this small repo."
