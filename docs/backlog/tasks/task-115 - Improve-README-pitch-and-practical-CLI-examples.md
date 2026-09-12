---
id: TASK-115
title: Improve README pitch and practical CLI examples
status: Done
assignee:
  - '@codex'
created_date: '2026-09-12 18:24'
updated_date: '2026-09-12 19:17'
labels: []
dependencies: []
ordinal: 96000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Explain XRAT through grounded benefits, use cases, and representative CLI output.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 README introduces practical workflows and key supported features without hype
- [x] #2 Commands and sample output are checked against local CLI or source and illustrative values are labeled
- [x] #3 README formatting and local links pass checks
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Read docs and CLI help; rewrite README with concise use cases and output; verify commands, formatting, and links.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Final README uses a friendly flag-free introduction with setup/TUI launch, upgrades, managed core installation, supported config add examples, import/list/test/update/rotation output, shell integration, and concise further-reading links. Table output uses fictional data; rotation and listing formats incorporate user-provided examples. All 8 add examples accepted by installed CLI in temporary storage. Import/add/list formats checked with temporary databases; local documentation links and formatting passed. Initial just fmt ci passed formatting and Clippy; tests hit sandbox socket permission restrictions, so CI is being rerun with required permissions.

Final validation: just fmt passed; full just ci passed with inherited HTTP/HTTPS/ALL proxy variables unset and local socket permissions available (813 library tests, 1 additional test). Default environment failures were sandbox and proxy interference. README-only and backlog-only changes are excluded by current CI and Docs push filters.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Reworked README into practical introductory workflows with verified mocked output and simple commands. Preserved engine-support boundaries and linked advanced usage. Tracked listing latency separately in TASK-116.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
