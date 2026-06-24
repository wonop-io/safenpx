# DB Contract Fail-Closed Policy

No database contract tests exist yet. When database-backed tests are introduced, missing database configuration must fail rather than skip silently.

Fix by requiring an explicit non-production database URL for DB contract tests.
