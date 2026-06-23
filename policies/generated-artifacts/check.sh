#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/generated-artifacts/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "${repo_root}" ]; then
  echo "Generated artifact policy skipped: not running inside a git worktree."
  exit 0
fi

cd "${repo_root}"
. policies/_lib/common.sh

base_ref="$(resolve_policy_base_ref "${1:-}")"
violations=""
checked_count=0

while IFS= read -r -d '' path; do
  [ -n "${path}" ] || continue
  checked_count=$((checked_count + 1))

  case "${path}" in
    bazel-*|bazel-*/*|bazel-bin|bazel-bin/*|bazel-out|bazel-out/*|bazel-testlogs|bazel-testlogs/*)
      violations="${violations}${path}: Bazel output must not be committed"$'\n'
      ;;
    target|target/*|*/target|*/target/*|dist|dist/*|*/dist|*/dist/*|coverage|coverage/*|*/coverage|*/coverage/*)
      violations="${violations}${path}: build or coverage output must not be committed"$'\n'
      ;;
    .DS_Store|*/.DS_Store|*.log|*.tmp|*.temp|*.profraw|*.profdata|*.lcov|*.db|*.sqlite|*.sqlite3)
      violations="${violations}${path}: local/generated artifact must not be committed"$'\n'
      ;;
  esac
done < <(changed_paths_z "${base_ref}")

if [ -n "${violations}" ]; then
  echo "Generated artifacts and local build outputs must not be committed." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Checked ${checked_count} changed path(s); no generated artifacts found."
