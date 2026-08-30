---
id: TASK-93
title: Support current Xray VMess and VLESS link fields
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 20:35'
updated_date: '2026-08-25 22:55'
labels:
  - bug
  - xray
  - share-link
  - generator
milestone: m-6
dependencies: []
references:
  - 'https://github.com/XTLS/Xray-core/discussions/716'
priority: high
ordinal: 55000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement the current official VMess/VLESS share-link proposal additions at the parser/generator boundary. Confirmed gaps are VMess encryption mapping, TLS ech/pcs/vcn, REALITY pqv, and finalmask fm; the query parser preserves most values but runtime generation leaves them unconsumed or expects internal names.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 VMess and VLESS encryption map to their official outbound fields and defaults
- [x] #2 ech, pcs, and vcn generate the documented TLS settings
- [x] #3 pqv generates mldsa65Verify for REALITY
- [x] #4 fm accepts percent-encoded JSON and generates streamSettings.finalmask without loss
- [x] #5 Malformed structured values fail with field-specific errors
- [x] #6 Official proposal examples and native Xray validation are covered by regression tests
- [x] #7 Duplicate query parameters are rejected as required by the official proposal
- [x] #8 Version-specific removed fields such as allowInsecure have explicit compatibility behavior
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Map current VMess/VLESS encryption, TLS, REALITY, and finalmask link fields with explicit compatibility errors.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verified: just fmt ci passed with Clippy warnings denied and 810 Rust tests plus the doctest. Native targeted configs passed Xray v26.3.27 and official v26.7.28. Cross-validated against tagged Xray-core source and Discussion 716. Documentation was updated in the reference and architecture guides.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Mapped official VMess and VLESS encryption, TLS ech/pcs/vcn, REALITY pqv, and finalmask fm fields with field-specific errors.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
