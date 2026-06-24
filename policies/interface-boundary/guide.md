# Interface Boundary Policy

Execution and unsafe boundaries need explicit design in `safe-npx`.

Fix by routing process execution, filesystem mutation, network access, and unsafe Rust through small reviewed modules with tests.
