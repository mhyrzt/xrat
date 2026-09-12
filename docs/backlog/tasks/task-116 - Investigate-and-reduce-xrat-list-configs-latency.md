---
id: TASK-116
title: Investigate and reduce xrat list configs latency
status: To Do
assignee: []
created_date: '2026-09-12 19:13'
labels:
  - performance
  - cli
dependencies: []
priority: medium
ordinal: 97000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
User reports that xrat list configs feels slightly slow on installed v0.19.1. The supplied listing includes a subscription with 98 configs, saved probe results, GeoIP columns, and an active runtime. No timings or cause have been established. Measure command startup and listing work, identify the bottleneck, and improve responsiveness without dropping output data. TASK-24 covers a related listing refactor but is not a prerequisite for this focused performance task.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Record reproducible baseline timings for cold and repeated runs on a representative dataset, including config and history sizes and GeoIP settings.
- [ ] #2 Identify the dominant source of latency with profiling or stage timings; distinguish bootstrap, database queries, enrichment, and rendering before choosing a fix.
- [ ] #3 Demonstrate reduced command latency with before/after measurements on the same fixture and environment.
- [ ] #4 Preserve config filters, refs, active state, saved metrics, GeoIP fields, and table/TSV/JSON output; run focused correctness checks for the changed path.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
