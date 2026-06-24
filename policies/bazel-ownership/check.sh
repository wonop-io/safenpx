#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/bazel-ownership/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "${repo_root}" ]; then
  echo "Bazel ownership policy skipped: not running inside a git worktree."
  exit 0
fi

cd "${repo_root}"
. policies/_lib/common.sh

base_ref="$(resolve_policy_base_ref "${1:-}")"
violations=""
checked_count=0

is_meaningful_source_path() {
  case "$1" in
    crates/*|policies/*|tools/*) ;;
    *) return 1 ;;
  esac

  case "$1" in
    *.rs|*.sh|*.bzl|BUILD|BUILD.bazel)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

has_bazel_owner() {
  local path="$1"
  local dir
  dir="$(dirname "${path}")"

  while [ "${dir}" != "." ] && [ "${dir}" != "/" ]; do
    if [ -f "${dir}/BUILD.bazel" ] || [ -f "${dir}/BUILD" ]; then
      return 0
    fi
    dir="$(dirname "${dir}")"
  done

  [ -f "BUILD.bazel" ] || [ -f "BUILD" ]
}

while IFS= read -r -d '' path; do
  [ -f "${path}" ] || continue
  is_meaningful_source_path "${path}" || continue
  checked_count=$((checked_count + 1))

  if ! has_bazel_owner "${path}"; then
    violations="${violations}${path}: no BUILD.bazel or BUILD owner found in folder ancestry"$'\n'
  fi
done < <(changed_paths_z "${base_ref}")

if [ -n "${violations}" ]; then
  echo "Changed meaningful source files must belong to a Bazel package." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Checked ${checked_count} changed source path(s); Bazel package ownership is present."
