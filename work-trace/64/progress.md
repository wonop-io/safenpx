# Issue 64 Progress

## 2026-06-26

- Moved issue #64 to `status:in-progress`.
- Created trace scaffold before implementation.
- Added an inspect-first POC guide covering human and JSON workflows, supported
  and unsupported command shapes, evidence limits, redaction, schema
  compatibility, reserved fields, and execution boundaries.
- Updated README entry points to use explicit M3 inspect commands and link to
  the inspect POC and JSON schema docs.
- Expanded JSON schema docs with redaction, authority-context, reserved-field,
  and no-run execution-boundary language.
- Fixed prior-commit review finding by moving `latest` from supported M3
  command shapes to unsupported floating-tag shapes until tag-race proof lands.
