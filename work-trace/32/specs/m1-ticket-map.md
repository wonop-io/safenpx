# M1 Ticket Map

## Source Deliverables

M1 from `docs/milestones.md` requires:

- package-spec parser with supported/unsupported matrix
- npm registry client for public package metadata
- exact-version resolution
- tarball download without lifecycle execution
- integrity verification and digest identity
- fixture manifest seed for parser, registry errors, integrity mismatch,
  malformed specs, and forwarded args
- unsupported-spec refusal messages that say what was rejected and whether
  anything was downloaded

## Existing Issues

- #2: parse supported exact-version package specs
- #3: resolve and verify root npm artifact identity
- #8: seed resolver and artifact fixture manifest

## Missing Issues To Create

- Define M1 resolver data contracts and reason vocabulary.
- Implement unsupported-spec refusal UX and exit behavior.
- Implement npm metadata client and stable registry error mapping.
- Implement tarball download with no-execution guarantees.
- Implement integrity verification and artifact digest identity.
- Add malformed-spec no-network tests.
- Add M1 inspect output wiring for human and JSON scaffold reports.
- Close M1 with an acceptance audit.

