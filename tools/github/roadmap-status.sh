#!/usr/bin/env bash
set -euo pipefail

REPO="${SAFENPX_REPO:-wonop-io/safenpx}"
OWNER="${SAFENPX_OWNER:-wonop-io}"

tools/github/ensure-gh-auth.sh

echo "Repository: ${REPO}"
echo
echo "Open issues"
gh issue list --repo "${REPO}" --state open --limit 30

echo
echo "Milestones"
gh api "repos/${REPO}/milestones?state=all" --jq '.[] | "- \(.title): \(.open_issues) open, \(.closed_issues) closed"'

echo
echo "Projects"
gh project list --owner "${OWNER}" --format json --limit 20 \
  --jq '.projects[] | select(.title | test("safe-npx|safenpx"; "i")) | "- #\(.number) \(.title)"'
