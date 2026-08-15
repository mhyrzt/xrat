---
id: TASK-63
title: Support sing-box GeoIP and Geosite routing through rule sets
status: To Do
assignee: []
created_date: '2026-08-15 11:20'
labels: []
dependencies: []
references:
  - 'https://sing-box.sagernet.org/configuration/rule-set/'
  - 'https://sing-box.sagernet.org/migration/#migrate-geoip-to-rule-sets'
priority: medium
ordinal: 23000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add faithful modern sing-box support for the global routing.direct.geosite, routing.direct.geoip, routing.block.geosite, and routing.block.geoip settings. Legacy sing-box GeoIP/Geosite route fields were removed, so this requires a rule-set and asset strategy rather than emitting deprecated fields.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Configured Geosite categories are translated to valid sing-box rule sets for Direct and Block behavior
- [ ] #2 Configured GeoIP categories are translated to valid sing-box rule sets for Direct and Block behavior
- [ ] #3 Rule-set assets have an explicit local or remote lifecycle with validation and actionable errors
- [ ] #4 Generated configurations pass sing-box check and preserve Direct-over-Block precedence
- [ ] #5 TUI help and configuration documentation describe sing-box rule-set requirements
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
