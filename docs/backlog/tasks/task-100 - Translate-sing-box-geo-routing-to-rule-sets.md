---
id: TASK-100
title: Translate sing-box geo routing to rule-sets
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 15:36'
updated_date: '2026-08-30 17:50'
labels:
  - sing-box
  - routing
  - rule-set
  - config-generation
milestone: m-7
dependencies:
  - TASK-98
references:
  - TASK-73
  - 'https://sing-box.sagernet.org/configuration/rule-set/'
  - 'https://github.com/yarikov/kvn-tui'
priority: medium
ordinal: 62000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Umbrella deliverable for replacing the current blanket rejection of routing.geosite and routing.geoip with sing-box 1.13 rule-set support. Split asset acquisition/format concerns from route-rule generation so each change remains reviewable.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Supported GeoIP and geosite selections generate documented local sing-box rule_set entries
- [ ] #2 Direct and block routing preserve ordering and fallback behavior
- [ ] #3 Missing, incompatible, or unavailable rule-set assets fail safely before process launch
- [ ] #4 Xray-only ext and negation syntax remains explicitly rejected unless an exact sing-box mapping exists
- [ ] #5 Representative rule-set configs pass sing-box check and have regression tests
- [ ] #6 Representative configurations pass sing-box v1.13.21 check
<!-- AC:END -->



## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
