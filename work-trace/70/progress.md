# Progress

- 2026-07-01: Started issue #70 with `just issue-start 70`.
- 2026-07-01: Read issue scope and M4 parent context after #69.
- 2026-07-01: Added trace scaffold before implementation.
- 2026-07-01: Added M4 exit-code constants for success, ask-required, deny,
  unsupported, inspection error, execution refused, and delegated execution
  failure.
- 2026-07-01: Migrated M1 report exit codes to derive from canonical policy
  decisions, preserving non-interactive ask-required exit `10`.
- 2026-07-01: Migrated M2 refusal exit codes to the same M4 table and kept
  compatibility aliases for existing M2 tests/callers.
- 2026-07-01: Updated M1/M2 fixture manifests and JSON golden fixtures from the
  interim `2/3/4/5` codes to the M4 `11/12/13/14` contract.
- 2026-07-01: Updated `docs/milestones.md` as the durable exit-code table.
- 2026-07-01: Added a contract test covering current allow, ask-required, deny,
  unsupported, inspection-error, execution-refused, and delegated-failure code
  values.
- 2026-07-01: `cargo test -p safe-npx` passed.
- 2026-07-01: Blue-team review found no actionable issues. Red-team review
  found M2 `non_interactive_stop` still exited `14` despite `ask_user`.
- 2026-07-01: Fixed M2 non-interactive stop to exit `10`, updated M2 fixture
  rows and stale diagnostic fixture data, and reran `cargo test -p safe-npx`
  successfully.
- 2026-07-01: Red-team and blue-team second-pass reviews found no remaining
  actionable issues.
- 2026-07-01: `just test` initially failed the file-size policy after the
  contract test pushed `report_tests.rs` over 500 lines.
- 2026-07-01: Moved the exit-code contract coverage into
  `report_exit_code_tests.rs`; `report_tests.rs` is back under the limit and
  `cargo test -p safe-npx` passed.
- 2026-07-01: Added `report_exit_code_tests.rs` to Bazel source wiring after
  `just test` caught the missing module in `bazel test //...`.
- 2026-07-01: `just test` passed.
