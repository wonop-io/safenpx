#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/db-contract-fail-closed/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

if find . -path './bazel-*' -prune -o -type f -name '*db*contract*test*' -print | grep -q .; then
  echo "Database contract tests must be reviewed for fail-closed behavior." >&2
  print_policy_guide_notice
  exit 1
fi

echo "No database contract tests present; policy not applicable yet."
