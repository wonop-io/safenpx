# Policy Completeness Policy

Every policy folder must include:

- `check.sh`
- `guide.md`
- executable permissions on `check.sh`
- references in `policies/BUILD.bazel`
- a failure message pointing to its guide

Fix by adding the missing wiring before changing policy behavior.
