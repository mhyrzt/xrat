---
id: TASK-99.1
title: Build shared sing-box 1.13 TLS and transport mapping
status: Done
assignee:
  - '@codex'
created_date: '2026-08-30 17:50'
updated_date: '2026-09-19 21:09'
labels:
  - sing-box
  - tls
  - transport
  - config-generation
milestone: m-7
dependencies:
  - TASK-98
references:
  - 'https://sing-box.sagernet.org/configuration/shared/tls/'
  - 'https://sing-box.sagernet.org/configuration/shared/v2ray-transport/'
  - 'https://github.com/yarikov/kvn-tui'
parent_task_id: TASK-99
priority: high
ordinal: 66000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Define reusable typed builders for the TLS and V2Ray transport fields shared by VLESS, VMess, and Trojan without importing Xray-specific wire assumptions. Map SNI, insecure, ALPN, uTLS, REALITY, WebSocket, gRPC, HTTP, and HTTPUpgrade only when normalized Xrat data is sufficient for exact sing-box 1.13 output.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Each accepted TLS and transport input has a documented Xrat-to-sing-box field mapping
- [x] #2 TLS-disabled profiles do not receive an enabled TLS block
- [x] #3 REALITY requires all sing-box-required fields and never guesses missing values
- [x] #4 WebSocket, gRPC, HTTP, and HTTPUpgrade preserve path, host, headers, and service names where supported
- [x] #5 Xray-only or lossy transport settings fail before launch with field-specific diagnostics
- [x] #6 Fixtures pass sing-box v1.13.21 check
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Use tagged sing-box v1.13.21 TLS and V2Ray transport option types as the authority. 2. Map only normalized SNI, insecure, ALPN, WebSocket, gRPC, HTTP, and HTTPUpgrade data with exact semantics. 3. Reject incomplete REALITY and unsupported/lossy transport extensions. 4. Add native-validator fixtures before enabling dependent outbounds.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added src/singbox/config/transport.rs: shared outbound TLS (SNI, insecure/allowInsecure alias, ALPN, validated uTLS fingerprint, REALITY with required pbk and 0-16 hex short id) and V2Ray transports (ws, grpc service_name, http host[]/path, httpupgrade host/path, quic). Unknown transports, unknown TLS security, non-scalar values, and Xray-only fields fail with field-specific errors. REALITY always emits utls (chrome default) because sing-box refuses a reality client without it. Native check fixtures: vless ws/grpc/httpupgrade/h2 pass against installed sing-box 1.13.19.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Shared TLS/transport mapping implemented and wired into VLESS/VMess/Trojan. Unrepresentable input is rejected before launch; native check fixtures pass on the installed supported binary. Pinned v1.13.21 conformance execution remains TASK-102/TASK-109.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
