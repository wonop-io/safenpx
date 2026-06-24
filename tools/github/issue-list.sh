#!/usr/bin/env bash
set -euo pipefail

REPO="${SAFENPX_REPO:-wonop-io/safenpx}"

tools/github/ensure-gh-auth.sh
gh issue list --repo "${REPO}" --state open --limit "${1:-50}"
