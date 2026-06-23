default: check

check: policy-checks fmt-check cargo-test coverage-check bazel-test
    @echo "All checks passed."

policy-checks BASE="":
    @./policies/check.sh {{ if BASE != "" { "--base " + BASE } else { "" } }}

fmt-check:
    cargo fmt --all -- --check

fmt:
    cargo fmt --all

cargo-test:
    cargo test

coverage-check:
    cargo llvm-cov --workspace --all-targets --fail-under-lines 80

bazel-test:
    bazel test //...

build:
    bazel build //...

test: check

tests: test

clean:
    bazel clean
    cargo clean
