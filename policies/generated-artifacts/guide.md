# Generated Artifacts Policy

Do not commit machine-local files, Bazel output symlinks, Cargo build output, coverage output, logs, or temporary files.

Fix by deleting the artifact, adding an ignore rule when appropriate, or regenerating the artifact in CI instead of storing it in git.
