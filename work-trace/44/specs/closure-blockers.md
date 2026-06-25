# M2 Closure Blockers

M2 treats root package metadata as static evidence. Lifecycle scripts and dependency declarations are not executed, installed, or resolved during blocker classification.

## Lifecycle Scripts

Install-time lifecycle scripts such as `preinstall`, `install`, `postinstall`, `prepare`, `prepublish`, `prepublishOnly`, and `prepack` block execution with `execution_refused` and `lifecycle_script_present`.

## Dependency Declarations

Runtime, optional, peer, and bundled dependency declarations block execution with `execution_refused` and `unsupported_closure` until dependency closure proof exists. Development dependencies are recorded as metadata but do not represent runtime install work for the M2 exact-version execution candidate.

Bundled dependency metadata is only a declaration in M2; it must not be assumed safe without fixture evidence proving bundled bytes.
