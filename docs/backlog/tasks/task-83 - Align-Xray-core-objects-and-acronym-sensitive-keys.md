---
id: TASK-83
title: Align Xray core objects and acronym-sensitive keys
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 20:35'
updated_date: '2026-08-25 22:55'
labels:
  - bug
  - xray
  - parser
  - stable
  - prerelease
milestone: m-6
dependencies: []
references:
  - 'https://github.com/XTLS/Xray-core/blob/v26.3.27/infra/conf/xray.go'
  - 'https://github.com/XTLS/Xray-core/blob/v26.7.28/infra/conf/xray.go'
priority: high
ordinal: 45000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Align root and feature objects in src/xray/parsing with official Xray-core v26.3.27 and v26.7.28 JSON contracts. Confirmed gaps include fakeDns casing, ObservatoryService, metrics tag/listen optionality, probeURL, FakeDNS and policy numeric widths, and prerelease env/geodata.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 fakeDns parses and serializes with the official key in strict and loose modes
- [x] #2 API, metrics, observatory, and FakeDNS fields match official requiredness, names, and value ranges
- [x] #3 Prerelease env and geodata objects are represented without breaking stable configs
- [x] #4 Fixtures accepted by Xray v26.3.27 and v26.7.28 round-trip without field loss
- [x] #5 Policy duration and counter fields cover the official unsigned ranges without accepting invalid negatives
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Align root/core feature objects and numeric types to the tagged v26.3.27/v26.7.28 structs; add strict round-trip fixtures; verify native and Rust checks.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verified: just fmt ci passed with Clippy warnings denied and 810 Rust tests plus the doctest. Native targeted configs passed Xray v26.3.27 and official v26.7.28. Cross-validated against tagged Xray-core source and Discussion 716. Documentation was updated in the reference and architecture guides.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Aligned fakeDns, API/metrics/observatory, env/geodata, and policy/core numeric schema fields to the audited Xray tags.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
