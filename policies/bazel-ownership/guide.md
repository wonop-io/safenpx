# Bazel Ownership Policy

Meaningful source files should live under a Bazel package.

Fix by adding a local `BUILD.bazel` or moving the file into an existing package. Keep ownership close enough that `bazel build //...` remains understandable.
