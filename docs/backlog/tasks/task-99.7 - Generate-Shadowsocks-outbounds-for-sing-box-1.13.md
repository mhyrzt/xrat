---
id: TASK-99.7
title: Generate Shadowsocks outbounds for sing-box 1.13
status: Done
assignee:
  - '@codex'
created_date: '2026-08-30 17:51'
updated_date: '2026-09-19 21:09'
labels:
  - sing-box
  - shadowsocks
  - outbound
milestone: m-7
dependencies:
  - TASK-98
references:
  - 'https://sing-box.sagernet.org/configuration/outbound/shadowsocks/'
parent_task_id: TASK-99
priority: high
ordinal: 75000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add Shadowsocks managed-runtime generation for the methods Xrat can import and sing-box 1.13 supports. Validate method/password requirements and explicitly handle SIP002 plugin options instead of silently dropping them.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Supported AEAD and AEAD-2022 methods map to exact sing-box method names
- [x] #2 Password/key requirements are validated per method family
- [x] #3 Unsupported legacy ciphers and plugin options fail before launch unless an exact supported mapping exists
- [x] #4 SIP002 and normalized database records generate equivalent output
- [x] #5 Representative fixtures pass sing-box v1.13.21 check
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
simple.rs validates the documented AEAD/2022 and legacy method set, base64 key length for 2022-blake3 methods (16/32 bytes), required password, and rejects TLS/SNI/host/path that imply an unsupported plugin. Native shadowsocks fixture passes.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Shadowsocks outbound generation implemented for documented methods with per-method key validation; unsupported ciphers and plugin-style fields fail before launch.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
