---
id: TASK-86
title: Correct WebSocket gRPC and HTTPUpgrade schemas
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 20:35'
updated_date: '2026-08-25 22:55'
labels:
  - bug
  - xray
  - parser
  - transport
milestone: m-6
dependencies: []
references:
  - >-
    https://github.com/XTLS/Xray-core/blob/v26.3.27/infra/conf/transport_internet.go
  - >-
    https://github.com/XTLS/Xray-core/blob/v26.7.28/infra/conf/transport_internet.go
priority: high
ordinal: 48000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Align typed transport objects with current Xray-core. Confirmed defects include camelCase gRPC timeout/window keys where Xray expects snake_case, missing gRPC authority and user_agent, missing WebSocket host/acceptProxyProtocol/heartbeatPeriod, and missing HTTPUpgrade acceptProxyProtocol.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every stable and prerelease gRPC JSON key parses and serializes with official spelling
- [x] #2 WebSocket and HTTPUpgrade objects represent every official field and no obsolete field is treated as current Xray schema
- [x] #3 Stable and prerelease transport fixtures round-trip without loss
- [x] #4 Regression tests distinguish snake_case gRPC keys from unsupported camelCase spellings
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Correct WebSocket, gRPC, and HTTPUpgrade field names and shapes; cover stable/prerelease serialization.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verified: just fmt ci passed with Clippy warnings denied and 810 Rust tests plus the doctest. Native targeted configs passed Xray v26.3.27 and official v26.7.28. Cross-validated against tagged Xray-core source and Discussion 716. Documentation was updated in the reference and architecture guides.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Corrected WebSocket, gRPC, and HTTPUpgrade fields and exact snake_case and acronym serialization.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
