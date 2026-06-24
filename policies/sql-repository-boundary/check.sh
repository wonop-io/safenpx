#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/sql-repository-boundary/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

if grep -RInE 'sqlx::query!?|query_as!?|diesel::' crates >/tmp/safenpx-sql-policy.$$ 2>/dev/null; then
  echo "Database query APIs are not yet part of safe-npx; add a repository boundary before using them." >&2
  print_policy_guide_notice
  echo >&2
  cat /tmp/safenpx-sql-policy.$$ >&2
  rm -f /tmp/safenpx-sql-policy.$$
  exit 1
fi
rm -f /tmp/safenpx-sql-policy.$$

echo "No database query boundary needed."
