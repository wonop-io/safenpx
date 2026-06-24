#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/release-evidence/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

if find .github/workflows -type f -maxdepth 1 -name '*release*' 2>/dev/null | grep -q .; then
  echo "Release workflows must preserve build, test, and provenance evidence before being enabled." >&2
  print_policy_guide_notice
  exit 1
fi

echo "No release workflow present; release evidence policy not applicable yet."
