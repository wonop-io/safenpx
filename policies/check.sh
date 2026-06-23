#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
Usage: policies/check.sh [--base REF]

Runs repository policy preflight in least-expensive-first order.
USAGE
}

base_ref=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --base)
      base_ref="${2:-}"
      shift 2
      ;;
    --base=*)
      base_ref="${1#--base=}"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "${repo_root}" ]; then
  echo "Policy preflight skipped: not running inside a git worktree."
  exit 0
fi

cd "${repo_root}"

POLICIES=(
  "Policy completeness:policies/policy-completeness/check.sh"
  "CI entry consistency:policies/ci-entry-consistency/check.sh"
  "Generated artifacts:policies/generated-artifacts/check.sh"
  "Local secret paths:policies/local-secret-paths/check.sh"
  "Secret hygiene:policies/secret-hygiene/check.sh"
  "Dependency sync:policies/dependency-sync/check.sh"
  "Documentation coverage:policies/documentation-coverage/check.sh"
  "Index coverage:policies/index-coverage/check.sh"
  "Bazel ownership:policies/bazel-ownership/check.sh"
  "Incomplete production Rust:policies/incomplete-production-rust/check.sh"
)

total="${#POLICIES[@]}"
index=0

for item in "${POLICIES[@]}"; do
  index=$((index + 1))
  label="${item%%:*}"
  script="${item#*:}"
  printf '\n[%02d/%02d] %s\n' "${index}" "${total}" "${label}"

  if [ -n "${base_ref}" ]; then
    "${script}" "${base_ref}"
  else
    "${script}"
  fi
done

echo
echo "Policy preflight passed."
