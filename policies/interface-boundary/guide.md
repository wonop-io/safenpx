# Interface Boundary Policy

Execution and unsafe boundaries need explicit design in `safe-npx`.

Fix by routing process execution, filesystem mutation, network access, and unsafe Rust through small reviewed modules with tests.

The reviewed process execution boundary is `crates/safe-npx/src/process_boundary.rs`.
Other production modules must not call `std::process::Command` or `Command::new`
directly.
