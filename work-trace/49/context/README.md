# Context

Issue #49 builds on the M2 closure contracts, artifact extraction, closure
blockers, bin selection, and selected-bin byte identity work.

The prototype is deliberately not a general `safe-npx --execute` path. It is a
local fixture harness for proving that direct execution can be constrained to a
verified root package with one selected bin, no lifecycle scripts, and no
dependency declarations.

