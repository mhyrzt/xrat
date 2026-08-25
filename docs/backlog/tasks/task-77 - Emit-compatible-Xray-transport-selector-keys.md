---
id: TASK-77
title: Emit compatible Xray transport selector keys
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 11:50'
updated_date: '2026-08-25 11:55'
labels:
  - bug
  - xray
dependencies: []
references:
  - 'https://github.com/mhyrzt/xrat/issues/2'
modified_files:
  - src/xray/config/types.rs
  - src/xray/config/stream.rs
  - src/xray/config/tuning.rs
  - src/xray/config/generator/tests.rs
priority: high
ordinal: 39000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
XRAT v0.18.1 emits only streamSettings.method, which Xray versions predating the network-to-method rename silently ignore and consequently default non-RAW transports to RAW. Generate both compatibility keys from one logical transport value so older cores consume network and newer cores consume method.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Generated Xray streamSettings emits both network and method with identical transport values
- [x] #2 XHTTP and representative non-RAW transport serialization has regression coverage
- [x] #3 Generated XHTTP configuration validates with the installed Xray core
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inspect every StreamSettings construction site and choose a representation that cannot produce conflicting selector values. 2. Implement dual-key serialization and update focused regression tests. 3. Run focused tests, formatting, and the full just fmt ci gate. 4. Record verification and finalize the task.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented dual transport-selector serialization by retaining network as XRAT's public transport value and adding a private method mirror at both StreamSettings construction sites. Added shared assertions covering RAW, XHTTP, gRPC, mKCP, and HTTPUpgrade, including the native XHTTP validator path. No user documentation change is required because this restores runtime compatibility without changing CLI behavior. Validation: focused generator suite passed 27 tests against installed Xray 26.3.27; just fmt ci passed rustfmt, Prettier, SQLite/PostgreSQL SQL formatting, strict Clippy, 787 library tests, and 1 binary test; git diff --check is clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Generated Xray streamSettings now includes identical network and method selectors, preserving compatibility across the upstream field rename. Regression coverage spans RAW and representative non-RAW transports, and all focused and repository-wide checks pass.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
