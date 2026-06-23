#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/dependency-sync/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "${repo_root}" ]; then
  echo "Dependency sync policy skipped: not running inside a git worktree."
  exit 0
fi

cd "${repo_root}"
. policies/_lib/common.sh

base_ref="$(resolve_policy_base_ref "${1:-}")"
changed_paths="$(changed_paths_z "${base_ref}" | tr '\0' '\n' | sort -u)"
violations=""
checked_count=0

path_changed() {
  printf '%s\n' "${changed_paths}" | grep -Fxq "$1"
}

if path_changed "MODULE.bazel" && [ ! -f "MODULE.bazel.lock" ]; then
  violations="${violations}MODULE.bazel changed but MODULE.bazel.lock is missing"$'\n'
fi

while IFS= read -r manifest; do
  [ -n "${manifest}" ] || continue
  checked_count=$((checked_count + 1))

  if [ ! -f "Cargo.lock" ]; then
    violations="${violations}${manifest}: Cargo manifest changed but Cargo.lock is missing"$'\n'
  fi

  if [ ! -f "MODULE.bazel.lock" ]; then
    violations="${violations}${manifest}: Cargo manifest changed but MODULE.bazel.lock is missing"$'\n'
  fi
done < <(printf '%s\n' "${changed_paths}" | grep -E '(^|/)Cargo\.toml$' || true)

if [ -n "${violations}" ]; then
  echo "Dependency manifest changes must keep Cargo and Bazel locks in sync." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Checked ${checked_count} Cargo manifest change(s); dependency metadata is synchronized."
