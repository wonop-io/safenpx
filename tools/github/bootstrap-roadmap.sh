#!/usr/bin/env bash
set -euo pipefail

REPO="${SAFENPX_REPO:-wonop-io/safenpx}"
OWNER="${SAFENPX_OWNER:-wonop-io}"
PROJECT_TITLE="${SAFENPX_PROJECT_TITLE:-safe-npx Roadmap}"

tools/github/ensure-gh-auth.sh

create_label() {
  local name="$1"
  local color="$2"
  local description="$3"

  gh label create "${name}" --repo "${REPO}" --color "${color}" --description "${description}" --force >/dev/null
}

create_milestone() {
  local title="$1"
  local due_on="$2"
  local description="$3"

  if gh api "repos/${REPO}/milestones?state=all" --jq '.[].title' | grep -Fxq "${title}"; then
    echo "Milestone exists: ${title}"
    return
  fi

  gh api -X POST "repos/${REPO}/milestones" \
    -f title="${title}" \
    -f due_on="${due_on}" \
    -f description="${description}" >/dev/null
  echo "Created milestone: ${title}"
}

create_issue_if_missing() {
  local title="$1"
  local labels="$2"
  local milestone_title="$3"
  local body="$4"

  if gh issue list --repo "${REPO}" --state all --search "${title} in:title" --json title --jq '.[].title' | grep -Fxq "${title}"; then
    echo "Issue exists: ${title}"
    return
  fi

  gh issue create --repo "${REPO}" --title "${title}" --label "${labels}" --milestone "${milestone_title}" "${project_issue_args[@]}" --body "${body}" >/dev/null
  echo "Created issue: ${title}"
}

create_project_field() {
  local project_number="$1"
  local name="$2"
  local data_type="$3"
  local options="${4:-}"

  if gh project field-list "${project_number}" --owner "${OWNER}" --format json --limit 100 --jq '.fields[].name' | grep -Fxq "${name}"; then
    echo "Project field exists: ${name}"
    return
  fi

  if [ "${data_type}" = "SINGLE_SELECT" ]; then
    gh project field-create "${project_number}" --owner "${OWNER}" --name "${name}" --data-type "${data_type}" --single-select-options "${options}" >/dev/null
  else
    gh project field-create "${project_number}" --owner "${OWNER}" --name "${name}" --data-type "${data_type}" >/dev/null
  fi

  echo "Created project field: ${name}"
}

echo "Bootstrapping labels for ${REPO}"
create_label "status:triage" "ededed" "Needs review and shaping."
create_label "status:ready" "c2e0c6" "Ready to be picked up."
create_label "status:in-progress" "1d76db" "Currently being worked on."
create_label "status:blocked" "d93f0b" "Blocked by a decision or external dependency."
create_label "type:feature" "0e8a16" "Feature or implementation work."
create_label "type:bug" "d73a4a" "Incorrect behavior."
create_label "type:security" "b60205" "Threat-model, policy, or security evidence work."
create_label "type:registry" "5319e7" "Audit registry work."
create_label "type:community" "fbca04" "Contributor, sponsor, or community work."
create_label "priority:p0" "b60205" "Critical path."
create_label "priority:p1" "d93f0b" "Important near-term work."
create_label "priority:p2" "fbca04" "Useful but not blocking."
create_label "track:cli" "1d76db" "CLI execution gate."
create_label "track:registry" "5319e7" "Audit registry."
create_label "track:policy" "006b75" "Policy engine."
create_label "track:security" "b60205" "Security model and corpus."
create_label "track:docs" "0075ca" "Documentation."
create_label "track:community" "fbca04" "Community and sponsorship."

echo "Bootstrapping milestones"
create_milestone "M0: Foundation" "2026-07-31T23:59:59Z" "Credible public scaffold, governance, planning, and contributor workflow."
create_milestone "M1: Package Resolution And Integrity" "2026-08-31T23:59:59Z" "Resolve npm specs to exact artifacts and verify integrity before execution."
create_milestone "M2: Dependency Graph And Evidence" "2026-09-30T23:59:59Z" "Dependency graph, lifecycle scripts, metadata anomalies, and fixture corpus."
create_milestone "M3: Policy Engine And Agent UX" "2026-10-31T23:59:59Z" "Allow/ask/deny policy, JSON schema, and agent/human UX."
create_milestone "M4: Audit Registry Alpha" "2026-11-30T23:59:59Z" "Audit records, dependency snowballing, and registry sync model."
create_milestone "M5: Public Beta And Ecosystem Adoption" "2026-12-31T23:59:59Z" "Public beta, contributor docs, sponsor readiness, and ecosystem guidance."

echo "Bootstrapping GitHub Project"
project_issue_args=()
project_number="$(gh project list --owner "${OWNER}" --format json --limit 100 --jq ".projects[] | select(.title == \"${PROJECT_TITLE}\") | .number" | head -n 1 || true)"
if [ -z "${project_number}" ]; then
  if project_json="$(gh project create --owner "${OWNER}" --title "${PROJECT_TITLE}" --format json 2>/dev/null)"; then
    project_url="$(printf '%s' "${project_json}" | jq -r .url)"
    project_number="$(printf '%s' "${project_json}" | jq -r .number)"
    echo "Created project: ${project_url}"
    project_issue_args=(--project "${PROJECT_TITLE}")
  else
    echo "Could not create project with gh. Create '${PROJECT_TITLE}' manually, then rerun this script." >&2
  fi
else
  echo "Project exists: ${PROJECT_TITLE} (#${project_number})"
  project_issue_args=(--project "${PROJECT_TITLE}")
fi

if [ -n "${project_number}" ]; then
  gh project edit "${project_number}" --owner "${OWNER}" --visibility PUBLIC --description "Six-month public roadmap for safe-npx." >/dev/null
  echo "Bootstrapping project fields"
  create_project_field "${project_number}" "Track" "SINGLE_SELECT" "CLI,Registry,Policy,Security,Docs,Community"
  create_project_field "${project_number}" "Priority" "SINGLE_SELECT" "P0,P1,P2,P3"
  create_project_field "${project_number}" "Target" "SINGLE_SELECT" "M0,M1,M2,M3,M4,M5"
  create_project_field "${project_number}" "Effort" "SINGLE_SELECT" "S,M,L,XL"
  create_project_field "${project_number}" "Risk" "SINGLE_SELECT" "Low,Medium,High"
  create_project_field "${project_number}" "Start date" "DATE"
  create_project_field "${project_number}" "Target date" "DATE"
fi

echo "Bootstrapping initial issues"
create_issue_if_missing "M0: Set up public GitHub roadmap workflow" "type:community,track:community,priority:p0,status:ready" "M0: Foundation" "Create labels, milestones, issue templates, planning scripts, and the roadmap project."
create_issue_if_missing "M1: Resolve npm package specs to exact versions" "type:feature,track:cli,priority:p0,status:ready" "M1: Package Resolution And Integrity" "Implement package spec parsing and dist-tag resolution without executing package code."
create_issue_if_missing "M1: Download and verify root package tarball integrity" "type:security,track:cli,priority:p0,status:ready" "M1: Package Resolution And Integrity" "Fetch the selected artifact, verify npm integrity, and report exact evidence."
create_issue_if_missing "M2: Build dependency graph without lifecycle execution" "type:feature,track:security,priority:p0,status:triage" "M2: Dependency Graph And Evidence" "Resolve dependency nodes and collect evidence without running install hooks or package binaries."
create_issue_if_missing "M3: Define stable agent JSON output schema" "type:feature,track:policy,priority:p1,status:triage" "M3: Policy Engine And Agent UX" "Design and test the JSON schema agents can use to stop and ask before execution."
create_issue_if_missing "M4: Define package-version audit record schema" "type:registry,track:registry,priority:p1,status:triage" "M4: Audit Registry Alpha" "Specify audit record fields for package artifact, integrity, timestamps, dependency links, and findings."
create_issue_if_missing "M5: Publish beta contributor and sponsor docs" "type:community,track:docs,priority:p2,status:triage" "M5: Public Beta And Ecosystem Adoption" "Prepare public beta docs, contributor guide, sponsor recognition, and ecosystem integration notes."

echo "Roadmap bootstrap complete."
