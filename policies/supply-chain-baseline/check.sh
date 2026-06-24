#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/supply-chain-baseline/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

violations=""
[ -f Cargo.lock ] || violations="${violations}Cargo.lock is missing"$'\n'
[ -f MODULE.bazel.lock ] || violations="${violations}MODULE.bazel.lock is missing"$'\n'
grep -Fq 'cargo llvm-cov --workspace --all-targets --fail-under-lines 80' .github/workflows/ci.yml || violations="${violations}CI must enforce coverage baseline"$'\n'

if [ -n "${violations}" ]; then
  echo "Supply-chain baseline wiring is incomplete." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Supply-chain baseline locks and CI gates are present."
