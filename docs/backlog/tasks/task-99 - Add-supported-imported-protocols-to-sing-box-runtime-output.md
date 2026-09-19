---
id: TASK-99
title: Deliver managed sing-box output for imported protocols
status: Done
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 15:36'
updated_date: '2026-09-19 21:09'
labels:
  - sing-box
  - outbound
  - protocols
  - config-generation
milestone: m-7
dependencies:
  - TASK-98
references:
  - TASK-73
  - 'https://sing-box.sagernet.org/configuration/outbound/'
  - 'https://github.com/yarikov/kvn-tui'
priority: medium
ordinal: 61000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Umbrella deliverable for adding managed sing-box runtime generation to protocols Xrat already imports and normalizes. Work is split into shared TLS/transport mapping and focused protocol tasks so each can be reviewed and validated independently. TUIC, ShadowTLS, AnyTLS, and SSH remain outside this task because Xrat does not yet model or import them.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Each newly supported protocol has an explicit field-by-field mapping from normalized Xrat data to current sing-box schema
- [x] #2 Incomplete or lossy mappings are rejected before launch rather than guessed or silently omitted
- [x] #3 TLS and transport blocks are shared only where their sing-box semantics are actually identical
- [x] #4 Representative configs for every added protocol pass sing-box check
- [x] #5 The user-facing support matrix distinguishes import support from managed sing-box runtime support
- [x] #6 Every added outbound passes native v1.13.21 validation
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
All seven subtasks are complete. Shared TLS/transport mapping plus strict VLESS/VMess/Trojan/HTTP/SOCKS/Shadowsocks outbounds; the test/probe runtime is engine-aware and spawns sing-box with a sing-box probe config. The compatibility doc records the per-protocol managed-runtime support matrix.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Managed sing-box output now covers every protocol xrat imports, with shared TLS/transport mapping, strict rejection of lossy input, engine-aware probes, and native check coverage.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
