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
- #32: plan resolver milestone execution

## Created Issues

- #33: define M1 resolver data contracts and reason vocabulary
- #34: implement unsupported-spec refusal behavior
- #35: implement npm metadata client and error mapping
- #36: download root tarball without execution
- #37: verify npm integrity and compute artifact identity
- #38: add malformed-spec no-network test harness
- #39: wire M1 evidence into human and JSON reports
- #40: audit and close resolver milestone acceptance

## Suggested Implementation Order

1. #33 Define data contracts and reason vocabulary.
2. #2 Implement exact-version parser.
3. #34 Implement unsupported-spec refusal behavior.
4. #38 Add malformed-spec no-network test harness.
5. #8 Seed fixture manifest and connect parser fixtures.
6. #35 Implement npm metadata client and error mapping.
7. #36 Download root tarball without execution.
8. #37 Verify integrity and compute artifact identity.
9. #3 Integrate root artifact resolution end to end.
10. #39 Wire M1 evidence into human and JSON reports.
11. #40 Audit and close M1.

## Notes

#2, #3, and #8 remain useful as broader integration tickets. The newly created
issues split out proof obligations that should not be hidden inside those broad
items.
