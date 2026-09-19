---
id: TASK-103.3
title: Map DNS strategy cache and fallback semantics
status: Done
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:51'
updated_date: '2026-09-19 21:09'
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
- [x] #1 UseIPv4 and UseIPv6 map to documented sing-box strategies
- [x] #2 UseIP and UseSystem are either mapped with proven equivalent behavior or rejected explicitly
- [x] #3 The first/final resolver policy is deterministic for empty and non-empty server lists
- [x] #4 Cache, fallback, and parallel-query settings never silently change meaning
- [x] #5 Boundary combinations pass native validation and unit tests assert semantic output
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
UseIPv4/UseIPv6 map to ipv4_only/ipv6_only; UseIP/UseSystem rejected; first server is the final resolver with a local fallback; disable_cache maps explicitly; disable_fallback and parallel-query=false are rejected rather than remapped. Along the way, discovered and fixed a missing route.default_domain_resolver that made any DNS-plus-TLS-server config fail sing-box check.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
DNS strategy, cache, fallback, and parallel-query semantics are mapped or explicitly rejected, and generated DNS configs now include the route-level default_domain_resolver required by sing-box 1.13.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
