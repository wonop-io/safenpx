#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/playbook-spec/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

if [ ! -d playbooks ]; then
  echo "No playbooks present; playbook specification policy not applicable yet."
  exit 0
fi

if [ ! -f playbooks/index.md ]; then
  echo "playbooks/index.md is required when playbooks exist." >&2
  print_policy_guide_notice
  exit 1
fi

if [ ! -f playbooks/repository/playbook-specification.md ]; then
  echo "playbooks/repository/playbook-specification.md is required." >&2
  print_policy_guide_notice
  exit 1
fi

missing=0
while IFS= read -r path; do
  name="$(basename "$path")"
  if [ "$name" = "index.md" ]; then
    continue
  fi

  first_line="$(sed -n '1p' "$path")"
  if [ "$first_line" != "---" ]; then
    echo "$path: missing YAML front matter." >&2
    missing=1
    continue
  fi

  for field in title domain summary created last_used last_updated; do
    if ! sed -n '2,20p' "$path" | grep -Eq "^${field}: "; then
      echo "$path: missing front matter field '$field'." >&2
      missing=1
    fi
  done
done < <(find playbooks -type f -name '*.md' | sort)

if [ "$missing" -ne 0 ]; then
  print_policy_guide_notice
  exit 1
fi

echo "Playbook specification metadata is valid."
