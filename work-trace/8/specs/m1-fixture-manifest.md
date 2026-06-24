# M1 Fixture Manifest Spec

## Format

The M1 fixture manifest should be machine-readable with one fixture per line.
Each fixture records the requested package/spec, expected result, expected
reason, expected exit code, forwarded args, and sentinel expectations where
relevant.

## Initial Coverage

- supported exact unscoped package
- supported exact scoped package
- supported exact package with forwarded args
- unsupported spec
- malformed spec
- registry error
- missing package
- missing version
- integrity mismatch

## Test Consumers

- Parser tests consume parser fixtures.
- No-network tests consume malformed/unsupported fixtures.
- Artifact-oriented tests consume registry and integrity failure seeds until
  the real registry and tarball clients exist.
