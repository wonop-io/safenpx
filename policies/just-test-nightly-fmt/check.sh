#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/just-test-nightly-fmt/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

if ! grep -Eq '^fmt-check:' justfile || ! grep -Fq 'cargo fmt --all -- --check' justfile; then
  echo "justfile must expose a formatter check." >&2
  print_policy_guide_notice
  exit 1
fi

echo "Formatter check is wired into justfile."
