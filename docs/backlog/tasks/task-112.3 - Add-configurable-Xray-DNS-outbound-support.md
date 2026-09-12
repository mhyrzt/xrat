---
id: TASK-112.3
title: Add configurable Xray DNS outbound support
status: To Do
assignee: []
created_date: '2026-09-02 11:02'
labels:
  - xray
  - dns
  - routing
  - config-generation
dependencies: []
references:
  - 'https://xtls.github.io/en/config/outbounds/dns.html'
  - 'https://xtls.github.io/en/config/routing.html'
  - src/xray/config/routing.rs
  - src/xray/parsing/protocols/outbounds.rs
documentation:
  - AGENTS.md
  - docs/src/05-reference/config-file.md
  - docs/src/06-architecture/config-generation.md
  - docs/src/03-features/runtime-management.md
parent_task_id: TASK-112
priority: medium
ordinal: 89000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Expose Xray's DNS outbound as a managed routing primitive. This is not an importable remote proxy node: it receives DNS traffic selected by routing and forwards, hijacks, drops, rejects, or rewrites it according to documented Xray settings. Add a typed Xrat configuration contract, validation, generated outbound and route integration, output, tests, and user documentation.

Required reading before implementation:
- AGENTS.md
- docs/src/05-reference/config-file.md
- docs/src/06-architecture/config-generation.md
- docs/src/03-features/runtime-management.md
- Xray DNS outbound documentation
- Xray routing documentation
- Existing top-level DNS and runtime routing builders

Keep the top-level Xray DNS resolver configuration conceptually separate from the DNS outbound. Do not present DNS as a subscription protocol or selectable remote server.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Users can configure a tagged Xray DNS outbound and route eligible DNS traffic to it through typed Xrat settings
- [ ] #2 The supported rewriteNetwork, rewriteAddress, rewritePort, userLevel, and ordered rule/action fields follow the targeted Xray schema
- [ ] #3 Invalid addresses, ports, networks, actions, rule combinations, or references fail before launch with field-specific errors
- [ ] #4 Generated DNS outbound and routing combinations pass native Xray validation for forward, hijack, drop, reject, and rewrite cases supported by Xrat
- [ ] #5 The feature composes with existing top-level DNS settings without overwriting or conflating them
- [ ] #6 Config reference, routing/runtime documentation, examples, and support matrix explain the DNS outbound's role and limitations
- [ ] #7 Focused configuration, generation, routing, serialization, and validation tests pass together with just fmt ci
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
