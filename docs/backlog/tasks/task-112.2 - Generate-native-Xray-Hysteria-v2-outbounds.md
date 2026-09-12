---
id: TASK-112.2
title: Generate native Xray Hysteria v2 outbounds
status: To Do
assignee: []
created_date: '2026-09-02 11:02'
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
- [ ] #1 Existing hysteria2 and hy2 imports can be generated for the Xray engine when every requested field has a documented Xray mapping
- [ ] #2 The generated outbound uses protocol hysteria version 2 plus method hysteria and complete hysteriaSettings required for authentication and transport
- [ ] #3 Unsupported sing-box-only or unrepresentable Hysteria2 options produce actionable pre-launch errors rather than being dropped
- [ ] #4 Explicit xray, explicit sing-box, and auto engine selection behavior is documented and covered by tests
- [ ] #5 Managed and probe configurations pass the targeted native Xray validator and existing sing-box Hysteria2 behavior remains covered
- [ ] #6 Protocol and runtime documentation clearly distinguish Xray-native Hysteria v2 from sing-box Hysteria2 support
- [ ] #7 Focused tests pass together with just fmt ci
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
