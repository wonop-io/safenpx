# Progress

- Moved issue #47 to `status:in-progress`.
- Created trace scaffold before implementation.
- Added a deterministic race matrix manifest and parser/evaluator covering
  metadata races, latest tag movement, cache poisoning, stale cache reuse,
  tarball integrity drift, pinned-delegation proof gaps, and exact-version pinning.
- Red-team review found stale cache reuse did not encode stale identity drift.
  Updated the fixture and evaluator so fail-closed race rows must carry identity
  drift, while exact-version pinning remains the no-drift allow-to-ask case.
