---
id: TASK-112.2
title: Generate native Xray Hysteria v2 outbounds
status: Done
assignee:
  - '@codex'
created_date: '2026-09-02 11:02'
updated_date: '2026-09-18 15:05'
labels:
  - protocols
  - xray
  - hysteria2
  - config-generation
dependencies: []
references:
  - 'https://xtls.github.io/en/config/outbounds/hysteria.html'
  - 'https://xtls.github.io/en/config/transports/hysteria.html'
  - 'https://v2.hysteria.network/docs/developers/URI-Scheme/'
  - TASK-101
  - src/xray/config/stream.rs
  - src/xray/config/outbound.rs
documentation:
  - AGENTS.md
  - docs/src/06-architecture/config-generation.md
  - docs/src/06-architecture/runtime-lifecycle.md
  - docs/src/05-reference/protocols.md
parent_task_id: TASK-112
priority: high
ordinal: 88000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add complete native Xray Hysteria v2 outbound generation for existing Hysteria2 nodes while retaining sing-box support. Xray separates the Hysteria proxy protocol from its QUIC transport, so the implementation must map both layers deliberately and validate compatibility rather than reusing the sing-box JSON shape.

Required reading before implementation:
- AGENTS.md
- docs/src/06-architecture/config-generation.md
- docs/src/06-architecture/runtime-lifecycle.md
- docs/src/05-reference/protocols.md
- Xray Hysteria outbound documentation
- Xray Hysteria transport documentation
- Official Hysteria2 URI scheme
- TASK-101 for the separate sing-box strictness work
- Existing Xray stream and outbound builders

Do not silently change the default auto-engine policy. Document and test whether native Xray Hysteria is selected explicitly, automatically, or only when its field set is fully representable.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Existing hysteria2 and hy2 imports can be generated for the Xray engine when every requested field has a documented Xray mapping
- [x] #2 The generated outbound uses protocol hysteria version 2 plus method hysteria and complete hysteriaSettings required for authentication and transport
- [x] #3 Unsupported sing-box-only or unrepresentable Hysteria2 options produce actionable pre-launch errors rather than being dropped
- [x] #4 Explicit xray, explicit sing-box, and auto engine selection behavior is documented and covered by tests
- [x] #5 Managed and probe configurations pass the targeted native Xray validator and existing sing-box Hysteria2 behavior remains covered
- [x] #6 Protocol and runtime documentation clearly distinguish Xray-native Hysteria v2 from sing-box Hysteria2 support
- [x] #7 Focused tests pass together with just fmt ci
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Verify current Xray Hysteria v2 outbound and transport fields against official docs/source. 2. Map representable XRAT Hy2 URI fields through the Xray outbound and stream builders with actionable rejection for unsupported fields. 3. Preserve auto/sing-box behavior, add generator and engine-selection tests, update protocol/config-generation/runtime docs. 4. Run focused tests and just fmt ci.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented native Xray hysteria outbound and hysteria transport for Hy2, with TLS SNI/ALPN/insecure/ECH mapping and patched-version certificate pinning. Xray stable v26.3.27 rejects pinSHA256 because of upstream GHSA-5wf9-h793-w73c; unrepresentable obfs/bandwidth/unknown options fail before launch. Parse auto still chooses sing-box; explicit parse xray and managed runtime.engine=xray use native Xray. Native xray run -test accepted probe and runtime configs. env NO_PROXY=127.0.0.1,localhost no_proxy=127.0.0.1,localhost rtk just fmt ci passed: 820 library tests, 1 binary test, clippy and formatting. Earlier default-proxy runs failed only local HTTP stub tests; the localhost bypass resolved them.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented native Xray Hysteria2 generation and managed runtime selection, preserved sing-box and parse-auto behavior, added strict option handling and docs, and validated with installed Xray 26.3.27 plus full fmt/CI (820+1 tests).
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
