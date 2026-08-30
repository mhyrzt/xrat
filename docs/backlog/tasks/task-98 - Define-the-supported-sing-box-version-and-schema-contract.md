---
id: TASK-98
title: Define the supported sing-box version and schema contract
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 15:36'
updated_date: '2026-08-30 17:52'
labels:
  - sing-box
  - config-generation
  - compatibility
milestone: m-7
dependencies: []
references:
  - TASK-73
  - 'https://sing-box.sagernet.org/configuration/'
  - 'https://github.com/SagerNet/sing-box'
  - 'https://github.com/yarikov/kvn-tui'
documentation:
  - docs/src/06-architecture/config-generation.md
priority: high
ordinal: 60000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Establish the exact sing-box compatibility baseline for generated managed-runtime configs. Xrat currently validates against whichever binary is installed while upstream schema behavior is evolving across 1.12, 1.13, and 1.14. Define the minimum/target versions, how future versions are handled, and an inventory of every generated top-level section, inbound, outbound, DNS server/rule, route rule, and experimental field. This is the contract used by the remaining milestone tasks.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The minimum and target sing-box versions are explicitly documented
- [ ] #2 A support matrix inventories every generated JSON shape and marks it supported, version-gated, or rejected
- [ ] #3 The matrix cites official sing-box documentation or upstream source for each version-sensitive field
- [ ] #4 The supported runtime range is stable sing-box >=1.13.0 and <1.14.0, with managed and CI conformance pinned to v1.13.21
- [ ] #5 Required detection and rejection behavior is specified for pre-1.13, 1.14, prerelease, malformed, and unavailable binaries
<!-- AC:END -->







## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Treat stable sing-box 1.13.x as the supported runtime line. 2. Pin conformance fixtures and CI validation to sing-box 1.13.21. 3. Accept installed stable versions >=1.13.0 and <1.14.0, while reporting the detected patch version. 4. Reject pre-1.13 and 1.14 prerelease/stable binaries until a deliberate compatibility audit updates the contract. 5. Inventory generated shapes against the v1.13.21 option structs, official documentation, and sing-box check.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Decision approved 2026-08-30: target the stable 1.13.x line and pin native validation to v1.13.21, currently GitHub's latest stable release. Local developer binary is v1.13.19. Treat 1.14 prereleases as unsupported until separately audited.
<!-- SECTION:NOTES:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
