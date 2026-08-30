---
id: TASK-87
title: Handle stable and prerelease mKCP schema divergence
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 20:35'
updated_date: '2026-08-25 22:55'
labels:
  - bug
  - xray
  - parser
  - generator
  - mkcp
milestone: m-6
dependencies: []
references:
  - >-
    https://github.com/XTLS/Xray-core/blob/v26.3.27/infra/conf/transport_internet.go
  - >-
    https://github.com/XTLS/Xray-core/blob/v26.7.28/infra/conf/transport_internet.go
priority: high
ordinal: 49000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make mKCP parsing and link-to-runtime generation explicit across Xray v26.3.27 and prerelease v26.7.28. Stable accepts congestion, buffers, header, and seed; prerelease replaces/removes part of that surface and adds cwndMultiplier and maxSendingWindow. The generator currently rejects stable seed/headerType while emitting fields removed by prerelease.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Stable mKCP fields accepted by v26.3.27 remain representable
- [x] #2 Prerelease cwndMultiplier and maxSendingWindow are represented
- [x] #3 Generation never silently emits a parameter that the selected compatibility target ignores
- [x] #4 Share-link errors identify version-specific unsupported fields rather than claiming current Xray universally removed them
- [x] #5 Native validation covers both audited Xray versions
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Model both audited mKCP schemas, select generation by compatibility target, and validate against both native binaries.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verified: just fmt ci passed with Clippy warnings denied and 810 Rust tests plus the doctest. Native targeted configs passed Xray v26.3.27 and official v26.7.28. Cross-validated against tagged Xray-core source and Discussion 716. Documentation was updated in the reference and architecture guides.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Separated stable and prerelease mKCP generation, added explicit removed-field errors, and validated targeted configs with both native binaries.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
