---
id: TASK-92
title: Make Xray strict and loose parsing semantically honest
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 20:35'
updated_date: '2026-08-25 22:55'
labels:
  - bug
  - xray
  - parser
  - roundtrip
milestone: m-6
dependencies: []
priority: high
ordinal: 54000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The strict parser only denies unknown root fields, while nested unknown fields are accepted; loose parsing drops unknown outbound and nested fields when serialized. Define and implement recursive strictness plus lossless loose round-tripping so parser modes match their documented behavior.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Strict mode rejects unknown fields at every typed schema level or is renamed and documented to reflect its actual scope
- [x] #2 Loose mode preserves unknown root and nested fields through parse and serialization
- [x] #3 Known stable and prerelease fields are typed rather than surviving only as opaque extras where validation is promised
- [x] #4 Tests cover unknown root, inbound, outbound, transport, security, DNS, and routing fields
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add lossless extension storage to typed schema objects. 2. Parse all modes through the lossless representation. 3. Recursively validate unknown fields for Strict mode with path-aware errors. 4. Add root and nested round-trip/strictness regression tests. 5. Run focused parser tests and record results.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verified: just fmt ci passed with Clippy warnings denied and 810 Rust tests plus the doctest. Native targeted configs passed Xray v26.3.27 and official v26.7.28. Cross-validated against tagged Xray-core source and Discussion 716. Documentation was updated in the reference and architecture guides.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented recursive path-aware strict validation and lossless loose-mode round trips for root and nested extensions.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
