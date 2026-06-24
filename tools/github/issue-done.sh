#!/usr/bin/env bash
set -euo pipefail

REPO="${SAFENPX_REPO:-wonop-io/safenpx}"
ISSUE="${1:?usage: issue-done.sh ISSUE_NUMBER}"

tools/github/ensure-gh-auth.sh
gh issue close "${ISSUE}" --repo "${REPO}" --reason completed --comment "Completed via Codex workflow."
