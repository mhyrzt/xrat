---
id: TASK-73
title: Audit sing-box runtime config generation mismatches
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-23 01:24'
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
- [ ] #1 All supported sing-box runtime config generators and setting mappings are inventoried and checked against the current sing-box schema/documentation.
- [ ] #2 Confirmed mismatches in outbounds, inbounds, transports, TLS, routing, DNS, or runtime options are fixed or explicitly documented as unsupported before launch.
- [ ] #3 Representative generated sing-box runtime configs pass the native sing-box validator where the binary is available.
- [ ] #4 Regression tests cover each corrected mismatch and ensure unsupported mappings fail safely without starting the process.
- [ ] #5 Sing-box-specific configuration-generation documentation reflects the verified support matrix and remaining limitations.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inventory sing-box runtime generation entry points, supported protocols, and app-to-sing-box mappings; exclude Xray. 2. Generate representative configs and compare them with current sing-box schema/docs and native check output. 3. Fix confirmed mismatches or add explicit pre-launch validation for unsupported settings. 4. Add regression/native-validator coverage and update the sing-box support matrix documentation. 5. Run focused tests and just fmt ci, then finalize the audit.
<!-- SECTION:PLAN:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
