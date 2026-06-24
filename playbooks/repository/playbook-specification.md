---
title: Playbook Specification
domain: repository
summary: Define required playbook metadata, indexes, and validation rules.
created: 2026-06-24
last_used: 2026-06-24
last_updated: 2026-06-24
---

# Playbook Specification

Use this playbook when creating or updating repository playbooks.

## Required Metadata

Every playbook under `playbooks/` must start with YAML front matter. Files named
`index.md` are navigation files and do not need playbook metadata.

Required fields:

- `title`: human-readable playbook title.
- `domain`: lowercase domain key, such as `repository`.
- `summary`: one short sentence for indexes.
- `created`: ISO date, `YYYY-MM-DD`.
- `last_used`: ISO date or `null`.
- `last_updated`: ISO date, `YYYY-MM-DD`.

## Index Rules

- `playbooks/index.md` lists every domain.
- Domain indexes list every local playbook.
- Indexes stay short and route readers to the smallest useful playbook.

## Quality Bar

A playbook should state when to use it, required inputs, the workflow, required
outputs, and verification. It should be concise enough to read before doing the
work.
