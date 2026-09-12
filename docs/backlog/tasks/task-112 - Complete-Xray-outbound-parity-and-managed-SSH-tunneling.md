---
id: TASK-112
title: Complete Xray outbound parity and managed SSH tunneling
status: To Do
assignee: []
created_date: '2026-09-02 10:59'
updated_date: '2026-09-02 11:00'
labels:
  - protocols
  - xray
  - ssh
  - runtime
  - initiative
dependencies: []
references:
  - 'https://xtls.github.io/en/config/outbounds/'
  - TASK-101
  - TASK-107
documentation:
  - AGENTS.md
  - docs/src/06-architecture/import-pipeline.md
  - docs/src/06-architecture/config-generation.md
  - docs/src/06-architecture/runtime-lifecycle.md
  - docs/src/05-reference/protocols.md
priority: medium
ordinal: 86000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Track the remaining operational protocol work identified by comparing Xray's current outbound catalog with Xrat's normalized import and managed-runtime pipelines. This initiative covers WireGuard, native Xray Hysteria v2, DNS and Loopback routing outbounds, and first-class managed SSH dynamic forwarding. Existing Freedom and Blackhole generation is already implemented; existing sing-box Hysteria2 correctness remains tracked by TASK-101.

Required reading before implementation of any child task:
- AGENTS.md and the repository guidelines it includes
- docs/src/06-architecture/import-pipeline.md
- docs/src/06-architecture/config-generation.md
- docs/src/06-architecture/runtime-lifecycle.md
- docs/src/05-reference/protocols.md
- The protocol-specific upstream documents attached to that child task

Scope rule: schema deserialization alone does not count as support. A protocol is complete only when accepted input is normalized and persisted without silent loss, generated for the intended engine, validated before launch, observable through runtime status/events, tested, and documented. Do not invent unofficial share-link formats without an explicit documented contract.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each missing outbound or tunnel capability is represented by a review-sized child task with explicit required reading
- [ ] #2 The support matrix distinguishes schema parsing, import support, persistence, Xray generation, sing-box generation, and managed runtime support
- [ ] #3 Unsupported or lossy inputs fail before process launch with actionable errors
- [ ] #4 Protocol work updates user documentation and passes focused tests plus just fmt ci
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
