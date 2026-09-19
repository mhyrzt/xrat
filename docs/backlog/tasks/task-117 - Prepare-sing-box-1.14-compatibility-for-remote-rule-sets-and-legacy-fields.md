---
id: TASK-117
title: Prepare sing-box 1.14+ compatibility for remote rule-sets and legacy fields
status: To Do
assignee:
  - '@codex'
created_date: '2026-09-19 23:20'
labels:
  - sing-box
  - compatibility
  - config-generation
dependencies: []
references:
  - 'https://sing-box.sagernet.org/migration/'
  - docs/src/06-architecture/singbox-compatibility.md
priority: high
ordinal: 98000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The compatibility contract now accepts any sing-box >=1.13.0 and warns outside the tested >=1.13.0,<1.15.0 range. Upstream deprecations become removals: the remote rule-set implicit HTTP client is deprecated in 1.14 and removed in 1.16, and legacy DNS address-filter fields such as ip_accept_any are deprecated in 1.14. Before those removals, generated configs must stop relying on the implicit HTTP client and legacy DNS fields so unbounded acceptance stays safe. Verified against a real sing-box 1.14.1 binary, which currently accepts all generated shapes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Remote rule-sets declare an explicit HTTP client for sing-box >=1.14 without breaking 1.13 output
- [ ] #2 Generated configs stop using legacy DNS address-filter fields or migrate them to response matching
- [ ] #3 The block outbound/outbound=block shape is migrated to action=reject when a supported minimum rises past its removal
- [ ] #4 Conformance fixtures run against the pinned 1.13.21 and a 1.14.x binary
- [ ] #5 The compatibility contract and deprecation-watch table reflect the final behavior
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Determine whether config generation needs the detected sing-box version threaded from preflight. 2. Emit http_clients/default_http_client only for >=1.14 and keep 1.13 output unchanged. 3. Replace ip_accept_any hosts routing with an exact-domain route rule or response matching. 4. Add 1.14 native fixtures. 5. Update the deprecation-watch table.
<!-- SECTION:PLAN:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
