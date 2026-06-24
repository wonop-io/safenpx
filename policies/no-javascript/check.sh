#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/no-javascript/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
[ -n "${repo_root}" ] || exit 0
cd "${repo_root}"
. policies/_lib/common.sh

base_ref="$(resolve_policy_base_ref "${1:-}")"
violations=""
checked_count=0

while IFS= read -r -d '' path; do
  checked_count=$((checked_count + 1))
  case "${path}" in
    *.js|*.jsx|*.mjs|*.cjs)
      violations="${violations}${path}: JavaScript source is not allowed in this Rust-first repository"$'\n'
      ;;
  esac
done < <(changed_paths_z "${base_ref}")

if [ -n "${violations}" ]; then
  echo "JavaScript files are blocked unless the repository policy changes explicitly." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Checked ${checked_count} changed path(s); no JavaScript source found."
