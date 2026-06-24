# Formatter Policy

The Wonop policy name is retained for parity. `safe-npx` uses stable `cargo fmt --all -- --check`.

Fix by keeping `fmt-check` in `justfile` and making `check` depend on it.
