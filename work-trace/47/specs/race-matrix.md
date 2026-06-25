# M2 Race Matrix

The M2 race matrix models state changes between inspection and execution
preparation. Every row is deterministic local data; no fixture depends on live
npm mutation.

## Race Classes

1. Metadata changed after inspection: fail closed with `metadata_changed`.
2. Tarball identity changed after inspection: fail closed with
   `cache_identity_mismatch` or the M1 `integrity_mismatch`, depending on where
   the mismatch is detected.
3. Dist-tag moved after inspection: unsupported for M2 exact-version execution;
   future `latest` support must pin before execution.
4. Cache poisoning: fail closed with `cache_identity_mismatch`.
5. Stale cache reuse: fail closed with `cache_identity_mismatch`.

## Execution Mechanism Implications

- `direct_extract` can proceed only when the prepared artifact identity equals
  the inspected identity.
- `pinned_delegation` remains blocked unless the delegated tool can prove it
  will execute the inspected bytes.
- `inspect_only_alpha` remains acceptable when race proof is incomplete.

