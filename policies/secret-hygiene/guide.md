# Secret Hygiene Policy

Changed text files must not contain private keys, API tokens, access keys, or long credential-looking assignments.

Fix by rotating any exposed secret, removing it from git, and documenting the required environment variable or GitHub secret name instead.
