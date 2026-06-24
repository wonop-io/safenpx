#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/file-size/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
[ -n "${repo_root}" ] || exit 0
cd "${repo_root}"
. policies/_lib/common.sh

base_ref="$(resolve_policy_base_ref "${1:-}")"
max_lines="${FILE_SIZE_MAX_LINES:-500}"
violations=""
checked_count=0

while IFS= read -r -d '' path; do
  [ -f "${path}" ] || continue
  case "${path}" in
    Cargo.lock|MODULE.bazel.lock|*.pdf|*.png|*.jpg|*.jpeg|*.gif|*.webp)
      continue
      ;;
  esac
  is_text_file "${path}" || continue
  checked_count=$((checked_count + 1))
  lines="$(wc -l < "${path}" | tr -d ' ')"
  if [ "${lines}" -gt "${max_lines}" ]; then
    violations="${violations}${path}: ${lines} lines exceeds ${max_lines}"$'\n'
  fi
done < <(changed_paths_z "${base_ref}")

if [ -n "${violations}" ]; then
  echo "Changed files should stay small enough to review." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Checked ${checked_count} text file(s); file sizes are within ${max_lines} lines."
