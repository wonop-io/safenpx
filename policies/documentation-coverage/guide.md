# Documentation Coverage Policy

Rust code should be understandable without archaeology. Document modules, public types, policy-facing data structures, and non-obvious functions.

This policy counts Rust item declarations under `crates/` and requires at least 80% of them to have a nearby `///` or `/** ... */` documentation comment. Prefer useful docs that explain intent, safety boundaries, and caller expectations.
