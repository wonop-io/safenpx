# M3 Ticket Map

M3 goal: produce useful human and machine evidence before any package code can
run.

## Ticket Set

| Issue | Title | Lane | Priority | Role |
| --- | --- | --- | --- | --- |
| #53 | M3: Plan inspect evidence ticket map | planning/docs | P0 | Planning ticket for this map. |
| #54 | M3: Wire inspect mode pipeline without execution | CLI | P0 | Orchestrates resolve/download/verify/extract/render and stops. |
| #55 | M3: Carry no-package-code canaries into inspect mode | security | P0 | Proves the real inspect path does not execute package code. |
| #56 | M3: Extract registry metadata evidence for inspect reports | artifact | P0 | Captures registry facts tied to resolved exact version. |
| #11 | M3: Extract root package evidence without execution | artifact | P0 | Captures verified tarball and `package.json` facts. |
| #57 | M3: Model inspect facts heuristics and decisions separately | policy | P0 | Shared evidence model for human and JSON renderers. |
| #58 | M3: Render human inspect report from shared evidence | CLI | P0 | Human report with facts, heuristics, decision, limits, and authority context. |
| #5 | M3: Define inspect JSON schema v0 for agents and CI | policy | P0 | Versioned machine schema and compatibility rules. |
| #59 | M3: Add deterministic inspect JSON golden fixtures | fixtures | P0 | Byte-stable JSON fixtures and schema compatibility guard. |
| #60 | M3: Model caller-declared source context | policy | P1 | Source context categories without brittle inference. |
| #12 | M3: Define authority-context redaction rules | policy/security | P0 | Redaction and authority-context categories. |
| #61 | M3: Define inspect decision receipt shape | policy | P1 | Receipt shape only; no approval-cache semantics yet. |
| #62 | M3: Reserve external evidence attestation and release-diff fields | policy | P1 | Reserved nullable fields, no hosted systems. |
| #63 | M3: Measure provisional inspect latency budgets | fixtures | P2 | Cold/warm inspect measurements without flaky CI. |
| #64 | M3: Document inspect-first POC workflow and limits | docs | P1 | README/docs update for M3 POC and limits. |
| #65 | M3: Audit and close inspect evidence milestone | planning/docs | P0 | Final M3 closeout audit. |

## Suggested Implementation Order

1. #54 inspect mode pipeline skeleton.
2. #55 no-package-code canaries on the real inspect path.
3. #56 registry metadata evidence.
4. #11 root package evidence.
5. #57 shared evidence model.
6. #60 source context.
7. #12 authority context and redaction.
8. #58 human report renderer.
9. #5 JSON schema v0.
10. #62 reserved nullable future fields.
11. #61 decision receipt shape.
12. #59 deterministic JSON golden fixtures.
13. #63 latency measurements.
14. #64 inspect-first POC docs.
15. #65 M3 closeout audit.

## Deliverable Coverage

| M3 deliverable | Issue coverage |
| --- | --- |
| Inspect mode that resolves, downloads, verifies, extracts evidence, and stops. | #54, #55 |
| Static extraction from registry metadata, tarball metadata, and `package.json`. | #56, #11 |
| Human report separating facts, provisional heuristics, decision, and authority context. | #57, #58, #12 |
| Versioned JSON schema with deterministic ordering and golden fixture outputs. | #5, #59 |
| Compatibility rule for additive fields, enum additions, and semantic changes. | #5, #59 |
| JSON top-level fields from M3 milestone. | #5, #57, #60, #12, #61, #62 |
| Nullable reserved fields for external evidence, attestations, and release diff. | #62, #5 |
| Decision receipt fields for local/shareable records. | #61 |
| Privacy and redaction rules for reports and JSON. | #12, #58, #59 |

## Evidence V0 Coverage

| Evidence item | Issue coverage |
| --- | --- |
| Requested command, selected bin, and forwarded args. | #54, #11, #57 |
| Source context. | #60, #57, #5, #58 |
| Resolved package identity and integrity verification. | #54, #56, #57 |
| Registry source and publish time. | #56, #12 |
| Publisher, maintainers, repository, license, and provenance fields when available. | #56, #11 |
| Package size, file count, binaries, and lifecycle scripts. | #11 |
| Dependency declarations labeled as declarations. | #11, #57 |
| Similar-name and unusual-shape heuristics as report-only signals. | #57, #58 |

## Acceptance Criteria Coverage

| M3 acceptance criterion | Issue coverage |
| --- | --- |
| Inspect mode never runs package binaries, lifecycle scripts, or dependency scripts. | #54, #55, #11 |
| JSON output is deterministic for fixture packages and has compatibility tests. | #5, #59 |
| Reports never print secret values, private registry tokens, or full sensitive environment details. | #12, #58, #59 |
| Authority context includes registry source, package scope, command intent, and cwd trust class with redacted/categories display. | #12, #60, #57 |
| Authority context examples include local terminal/CI, trusted/temp cwd, public/scoped registry, manual/agent. | #12, #60 |
| Redacted display output remains separate from hashed or canonicalized identity fields. | #12, #61 |
| Report plainly states what `safe-npx` catches and does not catch. | #58, #64 |
| Provisional inspect latency budgets are measured. | #63 |

## Later-Milestone Deferrals

- Broad execute mode remains M5.
- Approval cache semantics remain M5 and are not implemented by #61.
- Hosted audit records, attestations, and release diffs are reserved fields in
  M3 and implemented later.
- Full dependency closure execution remains beyond M3.
