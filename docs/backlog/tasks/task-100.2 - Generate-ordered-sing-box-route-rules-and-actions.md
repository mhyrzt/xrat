---
id: TASK-100.2
title: Generate ordered sing-box route rules and actions
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:51'
updated_date: '2026-08-30 17:52'
labels:
  - sing-box
  - routing
  - config-generation
milestone: m-7
dependencies:
  - TASK-98
  - TASK-100.1
references:
  - 'https://sing-box.sagernet.org/configuration/route/rule/'
  - 'https://sing-box.sagernet.org/configuration/route/rule_action/'
parent_task_id: TASK-100
priority: high
ordinal: 80000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Generate sing-box 1.13 domain, IP/CIDR, rule-set, direct, proxy, and block behavior with deterministic ordering. Decide when to use route-to-block outbound versus reject action based on the pinned schema and desired connection semantics.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 full, domain, keyword, regexp, IP, and CIDR inputs map to the documented match fields
- [ ] #2 Direct rules precede block rules and unmatched traffic uses proxy
- [ ] #3 Block behavior is explicitly tested for TCP and UDP semantics
- [ ] #4 Rule-set tags resolve to declared local rule-set objects
- [ ] #5 Negation, ext, dotless, and malformed values fail unless an exact mapping is documented
- [ ] #6 Representative routing combinations pass sing-box v1.13.21 check
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
