---
id: TASK-103.3
title: Map DNS strategy cache and fallback semantics
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:51'
labels:
  - sing-box
  - dns
  - config-generation
milestone: m-7
dependencies:
  - TASK-98
references:
  - 'https://sing-box.sagernet.org/configuration/dns/'
parent_task_id: TASK-103
priority: high
ordinal: 79000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Define exact sing-box 1.13 behavior for Xrat query_strategy, server order, final resolver, disable_cache, disable_fallback, parallel-query, and empty-server settings. Reject Xray-only behavior where no semantic equivalent exists.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 UseIPv4 and UseIPv6 map to documented sing-box strategies
- [ ] #2 UseIP and UseSystem are either mapped with proven equivalent behavior or rejected explicitly
- [ ] #3 The first/final resolver policy is deterministic for empty and non-empty server lists
- [ ] #4 Cache, fallback, and parallel-query settings never silently change meaning
- [ ] #5 Boundary combinations pass native validation and unit tests assert semantic output
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
