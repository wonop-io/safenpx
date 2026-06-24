#!/usr/bin/env bash
set -euo pipefail

REPO="${SAFENPX_REPO:-wonop-io/safenpx}"
ISSUE="${1:?usage: issue-start.sh ISSUE_NUMBER}"

tools/github/ensure-gh-auth.sh
USER="$(gh api user --jq .login)"

gh issue edit "${ISSUE}" --repo "${REPO}" --add-assignee "${USER}" --add-label "status:in-progress" --remove-label "status:ready" --remove-label "status:triage" || {
  gh issue edit "${ISSUE}" --repo "${REPO}" --add-assignee "${USER}" --add-label "status:in-progress"
}

gh issue comment "${ISSUE}" --repo "${REPO}" --body "Started work via Codex."
