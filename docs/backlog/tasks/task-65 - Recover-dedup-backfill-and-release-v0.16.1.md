---
id: TASK-65
title: Recover dedup backfill and release v0.16.1
status: In Progress
assignee:
  - '@codex'
created_date: '2026-08-15 13:17'
updated_date: '2026-08-15 13:23'
labels:
  - bug
  - release
dependencies: []
priority: high
ordinal: 25000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Fix the v0.15.0 startup failure when legacy configs collapse to the same v2 dedup key, harden the flaky process inspection test that blocked v0.16.0 publication, keep upgrade usable without database initialization, and publish a corrective v0.16.1 release.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 SQLite and PostgreSQL backfills preserve colliding legacy rows without violating the unique dedup key
- [x] #2 Backfill is transactional, idempotent, and recovers partially migrated databases
- [x] #3 Process inspection regression test waits for child exec with a bounded deadline
- [x] #4 xrat upgrade can run when database initialization fails
- [ ] #5 Regression tests and just ci pass
- [ ] #6 v0.16.1 is versioned, documented, tagged, and pushed without rewriting v0.16.0
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Make SQLite/PostgreSQL v1-to-v2 backfill transactional and assign deterministic preservation keys to canonical collisions. 2. Add duplicate, partial-state, and idempotency regressions. 3. Route upgrade before database bootstrap and add CLI execution coverage. 4. Stabilize the process inspection test with bounded polling. 5. Run just fmt ci, prepare v0.16.1 notes/version, commit in focused changes, tag, and push.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented transactional SQLite/PostgreSQL backfill with deterministic preservation keys for canonical collisions, bounded child-exec polling in the process inspection test, and database-independent upgrade dispatch. Focused tests pass. A SQLite backup of the actual failing database recovered from 29 v1 rows plus 114 migrated rows to 139 canonical v2 rows plus 4 preserved collision rows, retaining all 143 configs; a second pass was clean.
<!-- SECTION:NOTES:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
