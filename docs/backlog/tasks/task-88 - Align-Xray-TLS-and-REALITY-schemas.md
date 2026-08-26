---
id: TASK-88
title: Align Xray TLS and REALITY schemas
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 20:35'
updated_date: '2026-08-25 22:55'
labels:
  - bug
  - xray
  - parser
  - tls
  - reality
milestone: m-6
dependencies: []
references:
  - >-
    https://github.com/XTLS/Xray-core/blob/v26.3.27/infra/conf/transport_internet.go
  - >-
    https://github.com/XTLS/Xray-core/blob/v26.7.28/infra/conf/transport_security.go
priority: high
ordinal: 50000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Update full-JSON TLS and REALITY parsing for stable and prerelease Xray-core. Confirmed gaps include curvePreferences, pinnedPeerCertSha256, verifyPeerCertByName, ECH fields, the incorrect pinned certificate-chain field, and REALITY target/type/password/ML-DSA/fallback-limit fields.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 TLS fields use official names, shapes, and scalar/list semantics in both audited versions
- [x] #2 REALITY inbound and outbound fields, aliases, and ML-DSA fields round-trip without loss
- [x] #3 Removed or version-specific fields produce intentional compatibility behavior
- [x] #4 Fixtures validate successfully with the corresponding stable and prerelease Xray binaries
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Align TLS and REALITY fields with tagged sources and wire current share-link security parameters into generation.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verified: just fmt ci passed with Clippy warnings denied and 810 Rust tests plus the doctest. Native targeted configs passed Xray v26.3.27 and official v26.7.28. Cross-validated against tagged Xray-core source and Discussion 716. Documentation was updated in the reference and architecture guides.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Aligned TLS and REALITY full-JSON schemas and added ECH, certificate verification, ML-DSA, and compatibility-aware generation.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
