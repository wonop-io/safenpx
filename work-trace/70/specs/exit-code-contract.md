# Exit-Code Contract Spec

## Contract

M4 maps canonical decisions to stable process exit codes:

| Decision | Exit code | Meaning |
| --- | ---: | --- |
| `allow` | `0` | Inspection or execution succeeded. |
| `ask` | `10` | Human interaction is required in a non-interactive context. |
| `deny` | `11` | Policy denied the command. |
| `unsupported` | `12` | Input shape or requested command is unsupported. |
| `inspection_error` | `13` | Registry, tarball, extraction, or inspection work failed before execution. |
| `execution_refused` | `14` | Execution closure proof failed before package code ran. |
| delegated execution failure | `15` | Future delegated execution failed after safe-npx handed off execution. |

## Compatibility Notes

- Interactive ask remains compatible with existing inspect behavior until an
  explicit prompt/approval path exists; #69 reserves exit `10` for
  non-interactive ask-required stops.
- M2 execution refusal paths must converge on `14`.
- No package code should run while producing any non-zero M4 exit code.

## Tests

- Unit or CLI-level tests cover each current decision and exit code.
- Unsupported and malformed specs prove no network calls.
- JSON reports include exit codes matching process status.
- Human and JSON output modes share the same report-to-exit-code mapping.

