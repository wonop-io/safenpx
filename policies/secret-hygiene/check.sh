#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/secret-hygiene/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "${repo_root}" ]; then
  echo "Secret hygiene policy skipped: not running inside a git worktree."
  exit 0
fi

cd "${repo_root}"
. policies/_lib/common.sh

base_ref="$(resolve_policy_base_ref "${1:-}")"
violations=""
checked_count=0

scan_file() {
  local path="$1"

  case "${path}" in
    policies/secret-hygiene/*|Cargo.lock|MODULE.bazel.lock|*.lock|*.png|*.jpg|*.jpeg|*.gif|*.webp|*.pdf|*.zip|*.gz|*.mp4|*.mov|*.woff|*.woff2|*.ttf)
      return
      ;;
  esac

  is_text_file "${path}" || return

  awk -v path="${path}" '
    function allowed(line) {
      line = tolower(line)
      return line ~ /(example|placeholder|changeme|dummy|redacted|your[_-]?token|your[_-]?key|fake|sha512-)/
    }
    {
      if (allowed($0)) next
      if ($0 ~ /-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----/) printf "%s:%d: private key material\n", path, NR
      if ($0 ~ /(AKIA|ASIA)[A-Z0-9]{16}/) printf "%s:%d: AWS access key-like token\n", path, NR
      if ($0 ~ /gh[pousr]_[A-Za-z0-9_]{30,}/) printf "%s:%d: GitHub token-like secret\n", path, NR
      if ($0 ~ /sk-[A-Za-z0-9]{24,}/) printf "%s:%d: API token-like secret\n", path, NR
      if ($0 ~ /(api[_-]?key|secret|token|password|client[_-]?secret)[A-Za-z0-9_.-]*[[:space:]]*[:=][[:space:]]*["'"'"']?[A-Za-z0-9_\/+=.-]{24,}/) {
        printf "%s:%d: hard-coded credential-like assignment\n", path, NR
      }
    }
  ' "${path}"
}

while IFS= read -r -d '' path; do
  [ -n "${path}" ] || continue
  checked_count=$((checked_count + 1))
  file_violations="$(scan_file "${path}")"
  if [ -n "${file_violations}" ]; then
    violations="${violations}${file_violations}"$'\n'
  fi
done < <(changed_paths_z "${base_ref}")

if [ -n "${violations}" ]; then
  echo "Changed files contain credential-like material." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Scanned ${checked_count} changed path(s); no credential-like material found."
