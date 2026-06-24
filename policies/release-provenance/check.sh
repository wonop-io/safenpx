#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/release-provenance/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

if find . -path './bazel-*' -prune -o -type f \( -name '*release*.json' -o -name '*provenance*.json' \) -print | grep -q .; then
  echo "Release/provenance manifests require explicit provenance policy review." >&2
  print_policy_guide_notice
  exit 1
fi

echo "No release provenance manifests present; policy not applicable yet."
