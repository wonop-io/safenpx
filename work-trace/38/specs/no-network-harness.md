# No-Network Harness Spec

## Invariant

Malformed and unsupported package specs must stop before registry metadata or
tarball download work.

## Counters

The reusable harness should count:

- registry metadata attempts
- tarball download attempts

## Fixture Expectations

- malformed specs expect zero registry calls and zero tarball calls
- unsupported specs expect zero registry calls and zero tarball calls
- supported specs may proceed to resolver hooks in later M1 tickets
