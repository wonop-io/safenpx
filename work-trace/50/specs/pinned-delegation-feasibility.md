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

## M2 Recommendation

Pinned package-manager delegation is rejected for M2.

The rejection does not claim delegation can never be made safe. It says M2 does
not yet have enough proof to ask npm or another package manager to execute after
`safe-npx` inspection while preserving exact byte identity.

The first alpha should therefore choose either:

- direct execution from a verified local closure for the narrow root-only subset
  already proven by local fixtures, or
- `inspect_only_alpha` for command shapes outside that subset.

## Proof Gaps

The checked fixture manifest at
`crates/safe-npx/fixtures/delegation-feasibility-manifest.txt` records the M2
delegation gaps:

- metadata can be fetched again after inspection,
- floating tags or ranges can resolve to different package versions,
- `.npmrc`, environment, scope, or cwd can select a different registry,
- cache entries can be reused or mutated without `safe-npx` provenance,
- dependency installation can resolve and download bytes outside the verified
  closure,
- lifecycle scripts can run outside the inspected policy,
- generated shims are synthesized executable bytes not yet tied to inspected
  evidence,
- delegated bin lookup can differ from `safe-npx` selected-bin evidence,
- package-manager version, flags, and config can change behavior,
- delegated processes can inherit ambient environment and authority outside the
  verified closure.

Every row maps to `execution_refused` with reason `unsupported_closure`, backed
by local fixture classes only. No row requires live npm access or third-party
package execution.

## Follow-Up

If delegation is reconsidered after M2, follow-up issues must prove at least:

- package-manager execution from a local verified artifact without metadata
  re-resolution,
- deterministic dependency closure resolution and integrity verification,
- deterministic shim generation with byte identity,
- lifecycle-script policy and execution ordering inside the verified closure,
- registry and cache provenance agreement at execution time,
- environment and configuration isolation for delegated processes.
