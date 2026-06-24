#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/playbook-spec/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

if [ -d playbooks ]; then
  echo "Playbooks exist but no safe-npx playbook specification has been adopted yet." >&2
  print_policy_guide_notice
  exit 1
fi

echo "No playbooks present; playbook specification policy not applicable yet."
