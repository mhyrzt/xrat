---
id: TASK-100.1
title: Manage sing-box rule-set assets independently
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:51'
labels:
  - sing-box
  - rule-set
  - assets
milestone: m-7
dependencies:
  - TASK-98
references:
  - 'https://sing-box.sagernet.org/configuration/rule-set/'
  - 'https://github.com/yarikov/kvn-tui'
parent_task_id: TASK-100
priority: medium
ordinal: 78000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Define download, storage, freshness, format, and cleanup behavior for sing-box 1.13 rule-set assets without reusing incompatible Xray GeoIP/geosite database files. Integrate paths with existing application configuration and setup/upgrade flows.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Only documented source or binary sing-box rule-set formats are accepted
- [ ] #2 Rule-set files have deterministic per-user paths and atomic updates
- [ ] #3 Missing, stale, corrupt, and format-incompatible assets produce actionable diagnostics
- [ ] #4 Xray and sing-box assets cannot be confused by filename or path resolution
- [ ] #5 Setup, status, and documentation expose rule-set availability
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
