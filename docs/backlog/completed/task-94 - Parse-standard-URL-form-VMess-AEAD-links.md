---
id: TASK-94
title: Parse standard URL-form VMess AEAD links
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 20:36'
updated_date: '2026-08-25 22:55'
labels:
  - bug
  - xray
  - share-link
  - vmess
milestone: m-6
dependencies: []
references:
  - 'https://github.com/XTLS/Xray-core/discussions/716'
priority: high
ordinal: 56000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The official Xray share-link proposal defines VMess AEAD as vmess://uuid@host:port with query parameters, but xrat only accepts the legacy base64 JSON form. Add the current URL form while making legacy compatibility explicit.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Current URL-form VMess AEAD links parse according to Discussion 716
- [x] #2 Required host, port, UUID, encoding, case sensitivity, and duplicate-field rules are enforced
- [x] #3 Protocol, transport, TLS, and description fields map to the same normalized Node semantics as VLESS
- [x] #4 Legacy base64 JSON behavior is retained or intentionally deprecated with tests and documentation
- [x] #5 Parsed official examples generate Xray configs accepted by both audited versions where their fields overlap
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Add official URL-form VMess AEAD parsing with UUID and duplicate-field validation while retaining legacy JSON links.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verified: just fmt ci passed with Clippy warnings denied and 810 Rust tests plus the doctest. Native targeted configs passed Xray v26.3.27 and official v26.7.28. Cross-validated against tagged Xray-core source and Discussion 716. Documentation was updated in the reference and architecture guides.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added standard URL-form VMess AEAD parsing with required UUID, host, port, and duplicate rejection while retaining legacy base64 JSON.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
