#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/policy-completeness/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

find_policy_root() {
  local found=""

  if [ -d "policies" ] && [ -f "policies/BUILD.bazel" ]; then
    printf '%s\n' "policies"
    return
  fi

  for base in "${TEST_SRCDIR:-}" "${RUNFILES_DIR:-}" "${0}.runfiles"; do
    if [ -n "${base}" ] && [ -d "${base}" ]; then
      found="$(find -L "${base}" -type f -path "*/policies/BUILD.bazel" -print -quit)"
      if [ -n "${found}" ]; then
        dirname "${found}"
        return
      fi
    fi
  done

  echo "could not find policies/BUILD.bazel" >&2
  exit 2
}

policy_root="$(find_policy_root)"
build_file="${policy_root}/BUILD.bazel"
violations=""
checked_count=0

for dir in "${policy_root}"/*; do
  [ -d "${dir}" ] || continue
  name="$(basename "${dir}")"

  case "${name}" in
    _*|.*) continue ;;
  esac

  checked_count=$((checked_count + 1))

  if [ ! -f "${dir}/check.sh" ]; then
    violations="${violations}${name}: missing check.sh"$'\n'
    continue
  fi

  if [ ! -f "${dir}/guide.md" ]; then
    violations="${violations}${name}: missing guide.md"$'\n'
  fi

  if [ ! -x "${dir}/check.sh" ]; then
    violations="${violations}${name}: check.sh must be executable"$'\n'
  fi

  if ! grep -Fq "\"${name}/check.sh\"" "${build_file}"; then
    violations="${violations}${name}: policies/BUILD.bazel must reference ${name}/check.sh"$'\n'
  fi

  if ! grep -Fq "\"${name}/guide.md\"" "${build_file}"; then
    violations="${violations}${name}: policies/BUILD.bazel must reference ${name}/guide.md"$'\n'
  fi

  if ! grep -Fq "Policy guide: policies/${name}/guide.md" "${dir}/check.sh"; then
    violations="${violations}${name}: check.sh failure output must mention policies/${name}/guide.md"$'\n'
  fi
done

if [ -n "${violations}" ]; then
  echo "Policy folders must have complete check, guide, Bazel, and failure-guide wiring." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Checked ${checked_count} policy folder(s); all policy wiring is complete."
