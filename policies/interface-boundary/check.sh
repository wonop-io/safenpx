#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/interface-boundary/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
[ -n "${repo_root}" ] || exit 0
cd "${repo_root}"
. policies/_lib/common.sh

base_ref="$(resolve_policy_base_ref "${1:-}")"
violations=""
checked_count=0

while IFS= read -r -d '' path; do
  case "${path}" in
    crates/**/*.rs|crates/*.rs) ;;
    *) continue ;;
  esac
  [ -f "${path}" ] || continue
  checked_count=$((checked_count + 1))
  if grep -nE 'std::process::Command|Command::new|unsafe[[:space:]]*\{' "${path}" >/tmp/safenpx-interface.$$; then
    violations="${violations}$(sed "s#^#${path}:#" /tmp/safenpx-interface.$$)"$'\n'
  fi
  rm -f /tmp/safenpx-interface.$$
done < <(changed_paths_z "${base_ref}")

if [ -n "${violations}" ]; then
  echo "Changed Rust code reaches concrete execution/unsafe boundaries directly." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Checked ${checked_count} Rust file(s); interface boundaries are respected."
