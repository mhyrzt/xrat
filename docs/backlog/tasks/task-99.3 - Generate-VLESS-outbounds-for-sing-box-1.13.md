---
id: TASK-99.3
title: Generate VLESS outbounds for sing-box 1.13
status: Done
assignee:
  - '@codex'
created_date: '2026-08-30 17:51'
updated_date: '2026-09-19 21:09'
labels:
  - sing-box
  - vless
  - outbound
milestone: m-7
dependencies:
  - TASK-99.1
references:
  - 'https://sing-box.sagernet.org/configuration/outbound/vless/'
parent_task_id: TASK-99
priority: high
ordinal: 71000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add VLESS managed-runtime generation from normalized Xrat nodes. Cover UUID, flow, packet encoding, TLS/REALITY, and supported transports without carrying Xray-only field names or defaults into sing-box output.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Plain, TLS, and REALITY VLESS profiles generate only documented sing-box 1.13 fields
- [x] #2 XTLS Vision flow and packet encoding are preserved only when valid
- [x] #3 Supported WebSocket, gRPC, HTTP, and HTTPUpgrade profiles use the shared transport mapping
- [x] #4 Missing UUID, incomplete REALITY, and unsupported transport/security combinations fail before launch
- [x] #5 Representative fixtures pass sing-box v1.13.21 check
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Build strict VLESS v1.13 outbound from normalized fields, using the shared transport builder; validate plain/TLS/REALITY and native fixtures.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
vless.rs strict builder: UUID validation, flow only xtls-rprx-vision over direct TLS/REALITY, encryption must be none, packet_encoding packetaddr/xudp, shared TLS/transport. Regression tests for plain/reality/flow error/fingerprint/short id plus native fixtures.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
VLESS outbound generation is strict for sing-box 1.13 and native check passes for plain, TLS, REALITY, and transport profiles.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
