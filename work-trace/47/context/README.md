# Context

## Facts

- Issue #47 depends on M2 closure contracts, static extraction, and registry
  precedence from #42, #43, and #46.
- Live npm mutation must not be required to test race behavior.
- The fixture matrix should feed the later output wiring in #51.

## Assumptions

- Exact-version package specs are pinned by name, version, registry source, and
  artifact identity.
- Dist-tag movement is modeled as a future `latest` blocker rather than as an
  exact-version execution path.
- Cache and tarball races are represented by local identity mismatches.

