#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/incomplete-production-rust/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "${repo_root}" ]; then
  echo "Incomplete production Rust policy skipped: not running inside a git worktree."
  exit 0
fi

cd "${repo_root}"
. policies/_lib/common.sh

base_ref="$(resolve_policy_base_ref "${1:-}")"
violations=""
checked_count=0

is_production_rust_path() {
  case "$1" in
    crates/*.rs|crates/**/*.rs) ;;
    *) return 1 ;;
  esac

  case "$1" in
    */tests/*|*/test/*|*/benches/*|*/examples/*|*/fixtures/*|*_test.rs|*tests.rs)
      return 1
      ;;
  esac

  return 0
}

scan_file() {
  local path="$1"
  [ -f "${path}" ] || return

  awk -v path="${path}" '
    /^[[:space:]]*\/\// { next }
    /(^|[^A-Za-z0-9_])(todo|unimplemented|dbg|panic)![[:space:]]*\(/ {
      printf "%s:%d: incomplete or crash-only macro in production Rust: %s\n", path, NR, $0
    }
  ' "${path}"
}

while IFS= read -r -d '' path; do
  [ -n "${path}" ] || continue
  is_production_rust_path "${path}" || continue
  checked_count=$((checked_count + 1))

  file_violations="$(scan_file "${path}")"
  if [ -n "${file_violations}" ]; then
    violations="${violations}${file_violations}"$'\n'
  fi
done < <(changed_paths_z "${base_ref}")

if [ -n "${violations}" ]; then
  echo "Changed production Rust files contain incomplete or crash-only macros." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Scanned ${checked_count} changed production Rust file(s); no incomplete Rust macros found."
