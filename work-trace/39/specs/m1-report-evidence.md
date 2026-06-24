# M1 Report Evidence

## Behavior

- Parse the requested command intent before registry or artifact access.
- For unsupported or malformed specs:
  - Preserve the parser output.
  - Include the stable reason.
  - Report `downloaded=false`.
  - Do not attempt registry or tarball access.
- For supported exact-version specs:
  - Resolve npm metadata.
  - Download root tarball bytes.
  - Verify integrity.
  - Expose resolved package coordinates, registry source, tarball URL, verified
    integrity status, and digest identity.

## Failure Mapping

- Missing package, missing version, and registry/download failures remain ask /
  non-executable M1 report states with stable reasons.
- Integrity failures report `recommendation=deny` and
  `reason=integrity_mismatch`.

## Output Requirements

- JSON output uses stable snake_case field names.
- Human output is deterministic and names the same evidence as JSON when
  available.
- M1 output does not claim dependency graph, lifecycle script extraction,
  maintainer reputation, or later policy scoring is complete.

## Test Requirements

- Cover successful evidence output.
- Cover unsupported or malformed no-download output.
- Cover integrity mismatch denial.
- Keep network and tarball behavior stubbed in unit tests.
