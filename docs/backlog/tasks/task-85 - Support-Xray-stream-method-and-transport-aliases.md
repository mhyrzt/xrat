---
id: TASK-85
title: Support Xray stream method and transport aliases
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
ordinal: 47000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Bring StreamSettingsObject selectors and aliases in line with Xray-core. The parser models the deprecated root transport map as one stream object, omits stream address/port and prerelease method, rejects supported protocol aliases such as tcp, splithttp, mkcp, and websocket, and omits tcpSettings and splithttpSettings aliases.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 All stable and prerelease TransportProtocol names and aliases parse to a canonical representation
- [x] #2 Prerelease method is supported with the same precedence as Xray-core while stable network remains supported
- [x] #3 rawSettings/tcpSettings and xhttpSettings/splithttpSettings aliases preserve equivalent settings
- [x] #4 Tests cover every accepted selector and alias against both audited versions
- [x] #5 The deprecated root transport field has the official map shape and produces the same intentional Xray compatibility error
- [x] #6 Stream address and port fields round-trip where accepted by Xray-core
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Align transport selectors, aliases, root transport map, and stable/prerelease selector precedence; add regression coverage.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verified: just fmt ci passed with Clippy warnings denied and 810 Rust tests plus the doctest. Native targeted configs passed Xray v26.3.27 and official v26.7.28. Cross-validated against tagged Xray-core source and Discussion 716. Documentation was updated in the reference and architecture guides.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added canonical transport aliases, method-over-network precedence, stable and prerelease selectors, address and port, and the official root transport map shape.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
