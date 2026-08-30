---
id: TASK-84
title: Align Xray DNS and routing schemas
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 20:35'
updated_date: '2026-08-25 22:55'
labels:
  - bug
  - xray
  - parser
  - dns
  - routing
milestone: m-6
dependencies: []
references:
  - 'https://github.com/XTLS/Xray-core/blob/v26.7.28/infra/conf/dns.go'
  - 'https://github.com/XTLS/Xray-core/blob/v26.7.28/infra/conf/router.go'
priority: high
ordinal: 46000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Correct confirmed DNS and routing mismatches against stable and prerelease Xray-core. These include sourceIP/localIP/maxRTT/nonIPQuery acronym keys, domains/source and expectIPs compatibility aliases, NetworkList input forms, and signed types that reject valid unsigned Xray ranges.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Official acronym-sensitive keys parse and serialize exactly
- [x] #2 Official routing aliases and string-or-list network forms are accepted
- [x] #3 DNS expectedIPs and expectIPs compatibility forms are accepted and normalize predictably
- [x] #4 Numeric types cover every value accepted by both audited Xray versions
- [x] #5 Stable and prerelease native fixtures round-trip without dropping routing or DNS fields
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Correct DNS/routing aliases, acronym keys, string-list forms, and widths; add strict fixture coverage; verify checks.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verified: just fmt ci passed with Clippy warnings denied and 810 Rust tests plus the doctest. Native targeted configs passed Xray v26.3.27 and official v26.7.28. Cross-validated against tagged Xray-core source and Discussion 716. Documentation was updated in the reference and architecture guides.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Aligned DNS and routing acronym keys, compatibility aliases, NetworkList forms, rule type, and unsigned widths.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
