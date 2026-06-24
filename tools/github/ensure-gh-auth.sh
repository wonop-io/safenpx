#!/usr/bin/env bash
set -euo pipefail

if ! command -v gh >/dev/null 2>&1; then
  echo "GitHub CLI is required. Install it with: brew install gh" >&2
  exit 2
fi

if ! gh auth status >/dev/null 2>&1; then
  cat >&2 <<'EOF'
GitHub CLI is not authenticated.

Run:
  gh auth login
  gh auth refresh -s project

Or set GH_TOKEN with repo and project permissions.
EOF
  exit 2
fi
