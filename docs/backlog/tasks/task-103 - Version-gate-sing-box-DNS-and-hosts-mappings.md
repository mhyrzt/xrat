---
id: TASK-103
title: Version-gate sing-box DNS and hosts mappings
status: Done
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 15:36'
updated_date: '2026-09-19 21:09'
labels:
  - sing-box
  - dns
  - config-generation
  - compatibility
milestone: m-7
dependencies:
  - TASK-98
references:
  - TASK-73
  - 'https://sing-box.sagernet.org/configuration/dns/'
  - 'https://sing-box.sagernet.org/configuration/dns/server/hosts/'
  - 'https://github.com/yarikov/kvn-tui'
priority: high
ordinal: 65000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Umbrella deliverable for making every generated DNS and hosts mapping correct for the pinned sing-box 1.13.x contract. Split remote server/bootstrap behavior from hosts, strategy, and cache semantics.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 UDP, TCP, TLS, QUIC, HTTPS, HTTP3, local, and hosts outputs are either version-correct or rejected with a documented reason
- [x] #2 Domain-named remote DNS servers have a valid bootstrap resolver without dependency cycles
- [x] #3 Hosts routing uses the correct matcher for each supported sing-box version
- [x] #4 DNS strategy and cache settings preserve Xrat semantics or fail explicitly
- [x] #5 Native validation tests cover each supported DNS server and rule shape
- [x] #6 All supported DNS combinations pass sing-box v1.13.21 check
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
All three DNS subtasks are complete; native fixtures cover each server type, hosts rules, and the route-level resolver.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
sing-box DNS and hosts generation is version-correct for 1.13, with unsupported settings failing before launch.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
