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
  "Policy order:policies/policy-order/check.sh"
  "CI entry consistency:policies/ci-entry-consistency/check.sh"
  "Formatter wiring:policies/just-test-nightly-fmt/check.sh"
  "Conventional commits:policies/conventional-commits/check.sh"
  "Generated artifacts:policies/generated-artifacts/check.sh"
  "Local secret paths:policies/local-secret-paths/check.sh"
  "No JavaScript:policies/no-javascript/check.sh"
  "TypeScript exceptions:policies/typescript-exceptions/check.sh"
  "Secret hygiene:policies/secret-hygiene/check.sh"
  "SQL repository boundary:policies/sql-repository-boundary/check.sh"
  "Interface boundary:policies/interface-boundary/check.sh"
  "Dead code:policies/dead-code/check.sh"
  "Incomplete production Rust:policies/incomplete-production-rust/check.sh"
  "Migration timestamp:policies/migration-timestamp/check.sh"
  "Migrations immutable:policies/migrations-immutable/check.sh"
  "Dependency sync:policies/dependency-sync/check.sh"
  "Release inventory:policies/release-inventory/check.sh"
  "Release evidence:policies/release-evidence/check.sh"
  "Release provenance:policies/release-provenance/check.sh"
  "Protected promotions:policies/protected-promotions/check.sh"
  "DB contract fail-closed:policies/db-contract-fail-closed/check.sh"
  "Supply-chain baseline:policies/supply-chain-baseline/check.sh"
  "Supply-chain delta:policies/supply-chain-delta/check.sh"
  "Policy exceptions:policies/policy-exceptions/check.sh"
  "Playbook spec:policies/playbook-spec/check.sh"
  "Documentation coverage:policies/documentation-coverage/check.sh"
  "Index coverage:policies/index-coverage/check.sh"
  "Bazel ownership:policies/bazel-ownership/check.sh"
  "File size:policies/file-size/check.sh"
  "Replicated code:policies/replicated-code/check.sh"
  "Clean repo before tests:policies/clean-repo-before-tests/check.sh"
  "Changed-file coverage:policies/changed-file-coverage/check.sh"
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
