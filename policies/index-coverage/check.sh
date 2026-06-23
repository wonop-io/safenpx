#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/index-coverage/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "${repo_root}" ]; then
  echo "Index coverage policy skipped: not running inside a git worktree."
  exit 0
fi

cd "${repo_root}"
. policies/_lib/common.sh

base_ref="$(resolve_policy_base_ref "${1:-}")"
violations=""
checked_count=0

is_meaningful_source_path() {
  case "$1" in
    crates/*|policies/*|tools/*|.github/workflows/*) ;;
    *) return 1 ;;
  esac

  case "$1" in
    */docs/*|*.md|*.txt|*.json|*.lock|*.toml|*.yaml|*.yml)
      return 1
      ;;
    *.rs|*.sh|*.bzl|BUILD|BUILD.bazel)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

has_nearby_index() {
  local path="$1"
  local dir
  dir="$(dirname "${path}")"

  while [ "${dir}" != "." ] && [ "${dir}" != "/" ]; do
    if [ -f "${dir}/index.md" ]; then
      return 0
    fi
    dir="$(dirname "${dir}")"
  done

  [ -f "index.md" ]
}

while IFS= read -r -d '' path; do
  [ -f "${path}" ] || continue
  is_meaningful_source_path "${path}" || continue
  checked_count=$((checked_count + 1))

  if ! has_nearby_index "${path}"; then
    violations="${violations}${path}: no nearby index.md found in folder ancestry"$'\n'
  fi
done < <(changed_paths_z "${base_ref}")

if [ -n "${violations}" ]; then
  echo "Changed meaningful source folders must be covered by a nearby index.md." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Checked ${checked_count} changed source path(s); nearby indexes are present."
