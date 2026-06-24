# Dead Code Policy

Avoid adding `allow(dead_code)` or broad unused-code suppressions. They hide abandoned implementation paths in security-sensitive code.

Fix by deleting unused code, adding real call sites, or writing focused tests.
