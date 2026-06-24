#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/migration-timestamp/guide.md" >&2
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
  case "${path}" in
    *migrations/*.sql|*migration*.sql) ;;
    *) continue ;;
  esac
  checked_count=$((checked_count + 1))
  if ! basename "${path}" | grep -Eq '^[0-9]{14}[_-].+\.sql$'; then
    violations="${violations}${path}: migration filename must start with YYYYMMDDHHMMSS"$'\n'
  fi
done < <(changed_paths_z "${base_ref}")

if [ -n "${violations}" ]; then
  echo "Migration files must use timestamped filenames." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Checked ${checked_count} migration path(s); timestamps are valid."
