#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/release-inventory/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

if [ ! -f "crates/safe-npx/Cargo.toml" ] || [ ! -f "crates/safe-npx/BUILD.bazel" ]; then
  echo "The safe-npx release crate must remain present in Cargo and Bazel inventory." >&2
  print_policy_guide_notice
  exit 1
fi

echo "Release inventory contains the safe-npx crate."
