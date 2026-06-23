#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/documentation-coverage/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "${repo_root}" ]; then
  echo "Documentation coverage policy skipped: not running inside a git worktree."
  exit 0
fi

cd "${repo_root}"

threshold="${DOCUMENTATION_COVERAGE_THRESHOLD:-80}"
items=0
documented=0
violations=""

scan_file() {
  local path="$1"
  local result=""

  result="$(
    awk -v path="${path}" '
      function trim(line) {
        sub(/^[[:space:]]+/, "", line)
        sub(/[[:space:]]+$/, "", line)
        return line
      }

      function is_doc(line) {
        line = trim(line)
        return line ~ /^\/\/\// || line ~ /^\/\*\*/
      }

      function is_blank_or_attr(line) {
        line = trim(line)
        return line == "" || line ~ /^#\[/ || line ~ /^\/\// || line ~ /^\/\*/
      }

      function item_name(line) {
        line = trim(line)
        sub(/[{(;].*$/, "", line)
        return line
      }

      /^[[:space:]]*($|\/\/|#\[|\/\*)/ {
        if (is_doc($0)) {
          pending_doc = 1
        }
        next
      }

      /^[[:space:]]*(pub([[:space:]]*\([^)]*\))?[[:space:]]+)?(async[[:space:]]+)?(fn|struct|enum|trait|mod)[[:space:]]+[A-Za-z_][A-Za-z0-9_]*/ {
        total += 1
        if (pending_doc) {
          docs += 1
        } else {
          printf "%s:%d: missing item documentation: %s\n", path, NR, item_name($0) > "/dev/stderr"
        }
        pending_doc = 0
        next
      }

      {
        if (!is_blank_or_attr($0)) {
          pending_doc = 0
        }
      }

      END {
        printf "%d %d\n", total, docs
      }
    ' "${path}" 2>"/tmp/safenpx-doc-policy.$$"
  )"

  local file_violations
  file_violations="$(cat "/tmp/safenpx-doc-policy.$$")"
  rm -f "/tmp/safenpx-doc-policy.$$"

  if [ -n "${file_violations}" ]; then
    violations="${violations}${file_violations}"$'\n'
  fi

  local file_items file_docs
  file_items="${result%% *}"
  file_docs="${result##* }"
  items=$((items + file_items))
  documented=$((documented + file_docs))
}

while IFS= read -r -d '' path; do
  case "${path}" in
    */tests/*|*/test/*|*/fixtures/*|*/examples/*|*/benches/*)
      continue
      ;;
  esac
  scan_file "${path}"
done < <(find crates -type f -name '*.rs' -print0)

if [ "${items}" -eq 0 ]; then
  echo "No Rust items found for documentation coverage."
  exit 0
fi

coverage=$((documented * 100 / items))

if [ "${coverage}" -lt "${threshold}" ]; then
  echo "Rust documentation coverage is ${coverage}% (${documented}/${items}), below required ${threshold}%." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Rust documentation coverage is ${coverage}% (${documented}/${items}); required ${threshold}%."
