---
id: TASK-99.5
title: Generate VMess outbounds for sing-box 1.13
status: Done
assignee:
  - '@codex'
created_date: '2026-08-30 17:51'
updated_date: '2026-09-19 21:09'
labels:
  - sing-box
  - vmess
  - outbound
milestone: m-7
dependencies:
  - TASK-99.1
references:
  - 'https://sing-box.sagernet.org/configuration/outbound/vmess/'
parent_task_id: TASK-99
priority: high
ordinal: 73000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add VMess managed-runtime generation from normalized base64-JSON and URI imports. Map UUID, security cipher, alter_id, packet encoding, TLS, and supported transports using sing-box 1.13 semantics.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 AEAD VMess profiles emit valid UUID, security, alter_id, and packet encoding fields
- [x] #2 Supported TLS and transports use the shared typed mapping
- [x] #3 Legacy or unsupported cipher and transport combinations fail with actionable errors
- [x] #4 Both supported VMess import forms produce equivalent runtime output
- [x] #5 Representative fixtures pass sing-box v1.13.21 check
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Map normalized VMess UUID, alter ID, security, TLS, and transport to tagged v1.13.21 options, rejecting unrepresentable extensions; run native fixtures.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
vmess.rs maps UUID, security (auto/none/zero/aes-128-gcm/chacha20-poly1305/aes-128-ctr), alter_id from aid, packet_encoding, shared TLS/transport. Both base64-JSON and URL imports normalize to the same Node fields, so output is equivalent. Unknown cipher/encoding rejected.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
VMess outbound generation implemented with AEAD fields, packet_encoding, and shared TLS/transport; native check passes and unsupported ciphers fail before launch.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
