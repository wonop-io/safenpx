# Requirements Brief

## Acceptance Criteria

- Metadata changed after inspection fails closed with reason `metadata_changed`.
- Tarball/cache identity mismatch fails closed with reason
  `cache_identity_mismatch` or `integrity_mismatch` as appropriate.
- Dist-tag movement is covered as a future `latest` blocker; exact-version path
  remains pinned.
- No path re-resolves silently during execution preparation.
- Tests cover at least metadata race, tag race, cache race, and tarball identity
  race.

## Verification

- `just test`

