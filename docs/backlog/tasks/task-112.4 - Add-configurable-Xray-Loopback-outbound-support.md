---
id: TASK-112.4
title: Add configurable Xray Loopback outbound support
status: To Do
assignee: []
created_date: '2026-09-02 11:02'
labels:
  - xray
  - loopback
  - routing
  - config-generation
dependencies: []
references:
  - 'https://xtls.github.io/en/config/outbounds/loopback.html'
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
ordinal: 90000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Expose Xray's Loopback outbound as an advanced managed routing primitive for re-entering routing, including supported sniffing settings. Add a typed configuration contract, reference validation, generated outbound integration, loop-safety checks, tests, and documentation. Loopback is not an importable remote proxy protocol.

Required reading before implementation:
- AGENTS.md
- docs/src/05-reference/config-file.md
- docs/src/06-architecture/config-generation.md
- docs/src/03-features/runtime-management.md
- Xray Loopback outbound documentation
- Xray routing documentation
- Existing runtime routing builder and typed Loopback schema

The implementation must guard against obvious self-referential and deterministic routing cycles. It must not claim that all dynamic routing loops can be proven absent.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Users can configure a tagged Loopback outbound with inboundTag and supported sniffing settings through typed Xrat configuration
- [ ] #2 Generated Loopback outbounds can be referenced by routing, balancer fallback, and supported chaining fields without ad hoc raw JSON
- [ ] #3 Missing tags, duplicate tags, unknown references, direct self-reference, and deterministically detectable routing cycles fail before launch
- [ ] #4 Representative valid Loopback rerouting configurations pass the native Xray validator and behavioral tests confirm a second routing pass
- [ ] #5 CLI and documentation identify Loopback as an advanced routing primitive rather than a subscription protocol
- [ ] #6 Limitations of static cycle detection and safe example configurations are documented
- [ ] #7 Focused config, generation, reference-validation, and routing tests pass together with just fmt ci
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
