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

install-hooks:
    git config core.hooksPath .githooks
    @echo "Git hooks installed from .githooks"

roadmap-status:
    tools/github/roadmap-status.sh

roadmap-bootstrap:
    tools/github/bootstrap-roadmap.sh

issue-list:
    tools/github/issue-list.sh

issue-view ISSUE:
    tools/github/issue-view.sh "{{ISSUE}}"

issue-start ISSUE:
    tools/github/issue-start.sh "{{ISSUE}}"

issue-done ISSUE:
    tools/github/issue-done.sh "{{ISSUE}}"

test: check

tests: test

clean:
    bazel clean
    cargo clean
