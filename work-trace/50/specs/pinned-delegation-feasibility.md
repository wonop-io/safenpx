# Pinned Delegation Feasibility

Pinned delegation asks whether `safe-npx` can inspect one verified artifact and
then ask npm or another package manager to execute exactly those bytes.

For M2, delegation is acceptable only if every execution-time byte source is
proven to match inspect-time evidence:

- package metadata,
- root tarball bytes,
- selected executable bytes,
- generated shim bytes,
- dependency artifact bytes,
- lifecycle script behavior,
- registry source and cache entries,
- forwarded argv.

Any step that can re-resolve, install, mutate, synthesize executable bytes, or
read different registry/cache state without a matching proof must return
`execution_refused` or push the alpha toward `inspect_only_alpha`.

