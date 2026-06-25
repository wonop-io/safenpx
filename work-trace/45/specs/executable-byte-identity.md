# M2 Executable Byte Identity

M2 executable identity is inspect-time evidence. It records the exact bytes that
would run for the selected package bin, or refuses when those bytes cannot be
tied to the verified root artifact.

## Selected Bin

1. Bin selection provides a safe package-relative path.
2. The path is resolved under the verified extraction root.
3. Resolution fails closed if the canonical file path escapes the extraction
   root or is not a regular file.
4. The selected file is hashed before execution is considered.
5. The digest evidence is recorded as root-artifact executable identity.

## Generated Shims

Generated shims are not needed for the first direct-extract execution subset.
Until M2 explicitly models deterministic shim bytes, generated shim candidates
must fail closed with reason `unsupported_closure`.

## No Execution

Executable byte identity is built from filesystem metadata and file bytes only.
Tests must not invoke package binaries, generated shims, lifecycle scripts,
package managers, shells, or node.

