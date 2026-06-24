#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/protected-promotions/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

if grep -RInE 'environment:|workflow_dispatch|release' .github/workflows >/tmp/safenpx-promotions.$$ 2>/dev/null; then
  echo "Promotion/release workflows require protected environment review before use." >&2
  print_policy_guide_notice
  echo >&2
  cat /tmp/safenpx-promotions.$$ >&2
  rm -f /tmp/safenpx-promotions.$$
  exit 1
fi
rm -f /tmp/safenpx-promotions.$$

echo "No promotion workflows present; protected promotions policy not applicable yet."
