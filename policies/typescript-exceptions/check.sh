#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/typescript-exceptions/guide.md" >&2
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
    *.ts|*.tsx)
      violations="${violations}${path}: TypeScript requires an explicit policy exception"$'\n'
      ;;
  esac
done < <(changed_paths_z "${base_ref}")

if [ -n "${violations}" ]; then
  echo "TypeScript paths require an explicit exception." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Checked ${checked_count} changed path(s); no TypeScript exceptions needed."
