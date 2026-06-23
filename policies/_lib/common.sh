#!/usr/bin/env bash

find_repo_root() {
  git rev-parse --show-toplevel 2>/dev/null || true
}

resolve_policy_base_ref() {
  local requested="${1:-${POLICY_BASE_REF:-}}"
  local compare="${COMPARE_BRANCH:-main}"
  local candidate=""

  if [ -n "${requested}" ] && git rev-parse --verify --quiet "${requested}^{commit}" >/dev/null; then
    git merge-base "${requested}" HEAD 2>/dev/null || git rev-parse "${requested}^{commit}"
    return
  fi

  for candidate in "origin/${compare}" "${compare}" "origin/main" "main" "HEAD"; do
    if git rev-parse --verify --quiet "${candidate}^{commit}" >/dev/null; then
      git merge-base "${candidate}" HEAD 2>/dev/null || git rev-parse "${candidate}^{commit}"
      return
    fi
  done

  git rev-parse HEAD
}

changed_paths_z() {
  local base_ref="$1"
  {
    git diff --name-only -z --diff-filter=ACMRTUXB "${base_ref}" HEAD
    git diff --name-only -z --diff-filter=ACMRTUXB
    git diff --name-only -z --cached --diff-filter=ACMRTUXB
    git ls-files --others --exclude-standard -z 2>/dev/null || true
  } | sort -zu
}

is_text_file() {
  local path="$1"

  [ -f "${path}" ] || return 1
  LC_ALL=C grep -Iq . "${path}"
}
