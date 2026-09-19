---
id: TASK-73
title: Audit sing-box runtime config generation mismatches
status: Done
assignee:
  - '@codex'
created_date: '2026-08-23 01:24'
updated_date: '2026-09-19 21:09'
labels:
  - sing-box
  - config-generation
  - audit
dependencies: []
references:
  - docs/src/06-architecture/config-generation.md
documentation:
  - docs/src/06-architecture/config-generation.md
priority: high
ordinal: 35000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Audit only the sing-box managed runtime configuration path. Compare every supported generated JSON shape and setting mapping against the current sing-box schema/documentation and native validator behavior. Xray runtime generation is considered out of scope because it has already been validated. Correct confirmed sing-box mismatches and leave regression coverage and clear limitations for unsupported mappings.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 All supported sing-box runtime config generators and setting mappings are inventoried and checked against the current sing-box schema/documentation.
- [x] #2 Confirmed mismatches in outbounds, inbounds, transports, TLS, routing, DNS, or runtime options are fixed or explicitly documented as unsupported before launch.
- [x] #3 Representative generated sing-box runtime configs pass the native sing-box validator where the binary is available.
- [x] #4 Regression tests cover each corrected mismatch and ensure unsupported mappings fail safely without starting the process.
- [x] #5 Sing-box-specific configuration-generation documentation reflects the verified support matrix and remaining limitations.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inventory every generated sing-box runtime section against the v1.13.21 tagged option structs and official configuration documentation. 2. Confirm native validator behavior for representative configs. 3. Fix only confirmed generation mismatches or reject unsupported mappings before launch. 4. Add focused regressions, update the compatibility matrix, and record remaining delegated protocol work.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Inventoried every generated sing-box section against v1.13.21 option structs/docs and fixed confirmed mismatches: REALITY now forces uTLS, DNS configs emit route.default_domain_resolver, Shadowsocks methods/key lengths are validated, VMess security list corrected, packet_encoding supported, SOCKS UDP cannot be silently disabled, Clash API confined to loopback with port-collision checks, and the test/probe path now spawns sing-box with a sing-box probe config instead of feeding it an Xray config. Routing geosite/geoip and rule-set assets remain explicitly rejected (TASK-100).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the sing-box generation audit, fixed every confirmed mismatch across outbounds, TLS/transport, inbounds, DNS, and Clash API, added native and regression coverage, and updated the compatibility matrix. geosite/geoip rule-sets and the pinned conformance gate remain in TASK-100/TASK-102/TASK-109.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
