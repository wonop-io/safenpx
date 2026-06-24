#!/usr/bin/env bash
set -euo pipefail

REPO="${SAFENPX_REPO:-wonop-io/safenpx}"
ISSUE="${1:?usage: issue-view.sh ISSUE_NUMBER}"

tools/github/ensure-gh-auth.sh
gh issue view "${ISSUE}" --repo "${REPO}" --comments
