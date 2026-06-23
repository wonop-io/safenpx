#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/ci-entry-consistency/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

violations=""

require_file() {
  local path="$1"
  if [ ! -f "${path}" ]; then
    violations="${violations}${path}: missing required file"$'\n'
  fi
}

require_pattern() {
  local path="$1"
  local pattern="$2"
  local message="$3"

  if [ -f "${path}" ] && ! grep -Eq -- "${pattern}" "${path}"; then
    violations="${violations}${path}: ${message}"$'\n'
  fi
}

require_file ".github/workflows/ci.yml"
require_file "justfile"

require_pattern ".github/workflows/ci.yml" 'name:[[:space:]]*CI' "must be the canonical CI workflow"
require_pattern ".github/workflows/ci.yml" 'Policy Preflight' "must include a policy preflight job"
require_pattern ".github/workflows/ci.yml" '\./policies/check\.sh' "must run ./policies/check.sh"
require_pattern ".github/workflows/ci.yml" 'needs:[[:space:]]*policy' "test job must depend on policy preflight"
require_pattern ".github/workflows/ci.yml" 'cargo fmt --all -- --check' "must run cargo formatting"
require_pattern ".github/workflows/ci.yml" 'cargo test' "must run cargo tests"
require_pattern ".github/workflows/ci.yml" 'cargo llvm-cov --workspace --all-targets --fail-under-lines 80' "must enforce 80% coverage"
require_pattern ".github/workflows/ci.yml" 'bazel test //\.\.\.' "must run Bazel tests"

require_pattern "justfile" '^check:[[:space:]]*policy-checks' "check recipe must start with policy-checks"
require_pattern "justfile" '^policy-checks' "must expose a policy-checks recipe"
require_pattern "justfile" '\./policies/check\.sh' "policy-checks recipe must run ./policies/check.sh"
require_pattern "justfile" 'cargo llvm-cov --workspace --all-targets --fail-under-lines 80' "must enforce 80% coverage locally"

if [ -n "${violations}" ]; then
  echo "CI and local entrypoints must run policy preflight before expensive work." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "CI and local entrypoints run policy preflight before expensive work."
