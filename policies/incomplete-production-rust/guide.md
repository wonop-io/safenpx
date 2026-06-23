# Incomplete Production Rust Policy

Production Rust should not introduce `todo!`, `unimplemented!`, `dbg!`, or `panic!` macros.

Fix by returning a typed error, implementing the path, or isolating exploratory code under tests or fixtures.
