---
id: TASK-113
title: Bound retention of completed runtime session logs
status: Done
assignee:
  - '@codex'
created_date: '2026-09-11 21:53'
updated_date: '2026-09-11 21:58'
labels: []
dependencies: []
references:
  - 'https://github.com/mhyrzt/xrat/issues/3'
ordinal: 94000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
GitHub issue #3 reports 150 MiB of accumulated runtime logs. Automatically prune old completed-session stdout/stderr logs while preserving live sessions and recent diagnostics.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Cleanup bounds completed-session logs and preserves starting/running sessions and unrelated files
- [x] #2 Cleanup failures do not prevent normal runtime operations
- [x] #3 Regression tests and user documentation cover retention behavior
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add narrowly scoped completed-session log retention. 2. Run cleanup during application startup and before runtime launches. 3. Test retention and safety boundaries and document defaults.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented retention of the 10 newest completed sessions, protecting starting/running/stopping sessions and unknown files. Cleanup runs on AppContext startup and before connect/replacement launches. Exact filename matching and regular-file checks protect configs, custom files, directories, and symlinks. Validation: just ci passed (format, Clippy, 813 library tests and 1 binary test); git diff --check passed. PostgreSQL query implemented but no live PostgreSQL backend verification was performed. No user runtime files were manually cleaned.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Fixes issue #3 accumulation with best-effort completed-session stdout/stderr retention for Xray/V2Ray and sing-box. Documents defaults and limits. Active file sizes, daemon.log, custom logs, generated configs, and files without database sessions remain outside retention. Local implementation complete; not committed or released.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
