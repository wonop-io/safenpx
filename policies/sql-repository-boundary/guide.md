# SQL Repository Boundary Policy

`safe-npx` has no database layer yet. If one appears, SQL access must live behind repository modules.

Fix by designing the storage boundary before introducing query call sites.
