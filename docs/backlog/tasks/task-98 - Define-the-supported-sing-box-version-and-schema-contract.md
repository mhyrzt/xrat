---
id: TASK-98
title: Define the supported sing-box version and schema contract
status: Done
assignee:
  - '@codex'
created_date: '2026-08-30 15:36'
updated_date: '2026-09-19 23:21'
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
- [x] #1 The minimum and target sing-box versions are explicitly documented
- [x] #2 A support matrix inventories every generated JSON shape and marks it supported, version-gated, or rejected
- [x] #3 The matrix cites official sing-box documentation or upstream source for each version-sensitive field
- [x] #4 The supported runtime range is stable sing-box >=1.13.0 and <1.14.0, with managed and CI conformance pinned to v1.13.21
- [x] #5 Required detection and rejection behavior is specified for pre-1.13, 1.14, prerelease, malformed, and unavailable binaries
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Use sing-box v1.13.21 official documentation and tagged upstream option structs as the schema authority. 2. Inventory every XRAT-generated sing-box section and map it to a source or documentation contract. 3. Document the supported range, pinned conformance version, and rejection policy. 4. Add version-detection and launch-gate work only after the contract is explicit; validate each later task against this matrix.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Decision approved 2026-08-30: target the stable 1.13.x line and pin native validation to v1.13.21, currently GitHub's latest stable release. Local developer binary is v1.13.19. Treat 1.14 prereleases as unsupported until separately audited.

Documented the v1.13.21 contract and generated-shape matrix. Reviewed tagged v1.13.21 option structs and official documentation; local sing-box version is 1.13.19. Validation passed: rtk just fmt, rtk just mdbook-build, and rtk git diff --check.

Policy update (2026-09-20): the supported range is now unbounded above the minimum, sing-box >=1.13.0, with no <1.14.0 ceiling. Conformance fixtures cover >=1.13.0,<1.15.0 and the pinned target remains v1.13.21. A real 1.14.1 binary accepts all generated shapes. Versions outside the tested range are accepted with a warning; preflight still validates with the actual binary. See TASK-117 for the 1.16 removal readiness work.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Defined the stable >=1.13.0 and <1.14.0 runtime range, pinned v1.13.21 conformance target, field-source matrix, and rejection policy. Documentation formatting and book build pass; runtime enforcement follows in TASK-104.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
