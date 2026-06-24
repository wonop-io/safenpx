#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/supply-chain-delta/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

if [ "${SAFENPX_ENFORCE_CARGO_AUDIT:-0}" != "1" ]; then
  echo "RustSec delta gate is present but not enforced until an advisory baseline is introduced."
  exit 0
fi

if ! command -v cargo-audit >/dev/null 2>&1; then
  echo "cargo-audit is required when SAFENPX_ENFORCE_CARGO_AUDIT=1." >&2
  print_policy_guide_notice
  exit 1
fi

cargo audit --deny warnings
