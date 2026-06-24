#!/usr/bin/env bash
set -euo pipefail

print_policy_guide_notice() {
  echo "Policy guide: policies/policy-order/guide.md" >&2
  echo "Read the guide and follow it before retrying." >&2
}

violations=""

check_recipe_order() {
  local recipe_line
  recipe_line="$(grep -E '^check:' justfile || true)"
  if ! printf '%s\n' "${recipe_line}" | grep -Eq '^check:[[:space:]]+policy-checks[[:space:]]+fmt-check[[:space:]]+cargo-test[[:space:]]+coverage-check[[:space:]]+bazel-test'; then
    violations="${violations}justfile: check must run policy-checks, fmt, cargo tests, coverage, then Bazel"$'\n'
  fi
}

check_recipe_order

if [ -n "${violations}" ]; then
  echo "Local entrypoints must run cheap policy checks before expensive work." >&2
  print_policy_guide_notice
  echo >&2
  printf '%s' "${violations}" >&2
  exit 1
fi

echo "Policy order is correct."
