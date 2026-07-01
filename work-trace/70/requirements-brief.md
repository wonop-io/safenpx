# Requirements Brief

## Source

- GitHub issue #70.
- M4 parent issue #6.
- Current policy model and #69 non-interactive ask stop semantics.

## Scope

- Implement stable M4 exit-code mapping end to end.
- Keep human and JSON output modes consistent.
- Cover canonical policy decisions with tests.
- Document exit-code values in one durable place.

## Acceptance Criteria

- Every canonical decision has an exit-code test.
- Unsupported input exits `12` and malformed specs make no network calls.
- Integrity mismatch exits `11`.
- Unsupported closure exits `14`.
- Inspection errors exit `13`.
- Exit code values are documented in one durable place.

