# Playbook Specification Policy

Repository playbooks must be discoverable and carry required metadata.

Fix by ensuring:

- `playbooks/index.md` exists.
- `playbooks/repository/playbook-specification.md` exists.
- Every playbook markdown file under `playbooks/`, except `index.md`, starts
  with YAML front matter.
- Front matter includes `title`, `domain`, `summary`, `created`, `last_used`,
  and `last_updated`.
