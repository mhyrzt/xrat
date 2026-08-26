---
id: TASK-89
title: Align Xray sockopt and custom socket options
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 20:35'
updated_date: '2026-08-25 22:55'
labels:
  - bug
  - xray
  - parser
  - sockopt
milestone: m-6
dependencies: []
references:
  - >-
    https://github.com/XTLS/Xray-core/blob/v26.3.27/infra/conf/transport_internet.go
  - >-
    https://github.com/XTLS/Xray-core/blob/v26.7.28/infra/conf/transport_internet.go
priority: medium
ordinal: 51000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Complete SockoptObject against stable and prerelease Xray-core. Missing fields include tcpWindowClamp, penetrate, tcpMptcp, addressPortStrategy, and trustedXForwardedFor. Custom sockopt also lacks network and uses arbitrary JSON where Xray expects strings.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every socket option shared by the audited versions parses and serializes exactly
- [x] #2 Custom sockopt system/network/level/opt/value/type shapes match Xray-core
- [x] #3 Happy Eyeballs and acronym-sensitive fields retain official JSON spelling
- [x] #4 Stable and prerelease native fixtures round-trip without loss
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Align sockopt/custom socket option names, shapes, and acronym spelling; include strict fixture coverage.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verified: just fmt ci passed with Clippy warnings denied and 810 Rust tests plus the doctest. Native targeted configs passed Xray v26.3.27 and official v26.7.28. Cross-validated against tagged Xray-core source and Discussion 716. Documentation was updated in the reference and architecture guides.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed Sockopt and custom socket option schema coverage with exact official names and types.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
