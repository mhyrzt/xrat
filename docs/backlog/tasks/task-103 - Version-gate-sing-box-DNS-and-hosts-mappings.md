---
id: TASK-103
title: Version-gate sing-box DNS and hosts mappings
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 15:36'
updated_date: '2026-08-30 17:50'
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
- [ ] #1 UDP, TCP, TLS, QUIC, HTTPS, HTTP3, local, and hosts outputs are either version-correct or rejected with a documented reason
- [ ] #2 Domain-named remote DNS servers have a valid bootstrap resolver without dependency cycles
- [ ] #3 Hosts routing uses the correct matcher for each supported sing-box version
- [ ] #4 DNS strategy and cache settings preserve Xrat semantics or fail explicitly
- [ ] #5 Native validation tests cover each supported DNS server and rule shape
- [ ] #6 All supported DNS combinations pass sing-box v1.13.21 check
<!-- AC:END -->



## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
