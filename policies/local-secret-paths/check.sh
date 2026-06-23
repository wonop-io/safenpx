#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/local-secret-paths/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "${repo_root}" ]; then
  echo "Local secret path policy skipped: not running inside a git worktree."
  exit 0
fi

cd "${repo_root}"
. policies/_lib/common.sh

base_ref="$(resolve_policy_base_ref "${1:-}")"
violations=""
checked_count=0

is_blocked_path() {
  local path="$1"
  local base
  base="$(basename "${path}")"

  case "${path}" in
    .cargo/credentials|.cargo/credentials.toml|*/.cargo/credentials|*/.cargo/credentials.toml)
      return 0
      ;;
  esac

  case "${base}" in
    .env|.env.*|.npmrc|.netrc|.secret|.secret.*|.secrets|.secrets.*|secrets.env|*.secret.env|*.secrets.env|*.p12|*.pfx|*.pem|*.key|id_rsa|id_ed25519|known_hosts)
      return 0
      ;;
  esac

  return 1
}

while IFS= read -r -d '' path; do
  [ -n "${path}" ] || continue
  checked_count=$((checked_count + 1))
  if is_blocked_path "${path}"; then
    violations="${violations}${path}: secret/config-shaped files must not be committed"$'\n'
  fi
done < <(changed_paths_z "${base_ref}")

if [ -n "${violations}" ]; then
  echo "Changed paths include local secret or machine-specific config filenames." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Checked ${checked_count} changed path(s); no local secret/config filenames found."
