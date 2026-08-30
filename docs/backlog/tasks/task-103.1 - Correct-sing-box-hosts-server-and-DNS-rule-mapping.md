---
id: TASK-103.1
title: Correct sing-box hosts server and DNS rule mapping
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:51'
labels:
  - sing-box
  - dns
  - hosts
  - routing
milestone: m-7
dependencies:
  - TASK-98
references:
  - 'https://sing-box.sagernet.org/configuration/dns/server/hosts/'
parent_task_id: TASK-103
priority: high
ordinal: 76000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Verify Xrat's use_system_hosts and static dns.hosts translation against sing-box 1.13 hosts-server and DNS-rule semantics. Preserve exact host keys and scalar/list IP values while preventing broad hosts routing from changing unrelated answers.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 System hosts are enabled only when use_system_hosts is true
- [ ] #2 Static predefined hosts work without implicitly reading system host files when disabled
- [ ] #3 Exact host rules route only intended queries to the hosts server
- [ ] #4 Unsupported Xray host patterns and non-IP values fail with field-specific errors
- [ ] #5 System-only, predefined-only, and combined fixtures pass sing-box v1.13.21 check
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
