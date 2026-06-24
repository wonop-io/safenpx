#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/policy-exceptions/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

violations=""

while IFS= read -r path; do
  [ -n "${path}" ] || continue
  if ! grep -Eq 'reason|expires|owner|justification' "${path}"; then
    violations="${violations}${path}: exception files must include reason/owner/expiry context"$'\n'
  fi
done < <(find policies -type f \( -name '*allowlist*' -o -name '*exceptions*' \) -print)

if [ -n "${violations}" ]; then
  echo "Policy exceptions must be narrow and reasoned." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "No unreasoned policy exceptions found."
