# Execution Closure Contracts

## Contract Shape

- `ExecutionClosureEvidence` is an inspect-time evidence object, not permission
  to run package code.
- The root artifact is represented by the existing `ArtifactIdentity`.
- Selected binaries are represented separately from generated shims.
- Lifecycle scripts are represented as detected metadata, not executed steps.
- Dependency declarations are represented separately from verified dependency
  artifact identities.
- Cache and registry sources are represented as evidence inputs to later proof
  tickets.
- Command identity records the requested spec and forwarded arguments exactly as
  classified by the CLI.

## Reason Vocabulary

M2 extends the existing reason vocabulary with:

- `ambiguous_bin`
- `missing_bin`
- `lifecycle_script_present`
- `unsupported_closure`
- `metadata_changed`
- `cache_identity_mismatch`
- `registry_precedence_mismatch`
- `shim_identity_mismatch`
- `non_interactive_stop`

## Refusal Mapping

- M2 closure proof failures map to `ClosureDecision::ExecutionRefused`.
- Known unsafe evidence maps to `ClosureDecision::Deny`.
- Missing or unreliable inspection data maps to `ClosureDecision::InspectionError`.
- Unsupported command shapes map to `ClosureDecision::Unsupported`.
- Human intervention maps to `ClosureDecision::Ask` until M4 defines broader
  policy semantics.

## Safety Boundaries

- These contracts must not claim root tarball verification is sufficient to
  execute.
- These contracts must not model dependency declarations as verified artifacts.
- These contracts must not imply package manager delegation is safe.
