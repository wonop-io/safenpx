#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/changed-file-coverage/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

if [ "${CHANGED_FILE_COVERAGE_ENABLED:-1}" != "1" ]; then
  echo "Changed-file coverage disabled by CHANGED_FILE_COVERAGE_ENABLED."
  exit 0
fi

if command -v cargo-llvm-cov >/dev/null 2>&1; then
  cargo llvm-cov --workspace --all-targets --fail-under-lines 80
else
  echo "cargo-llvm-cov is not installed; CI enforces the 80% coverage gate."
fi
