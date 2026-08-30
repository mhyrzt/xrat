---
id: TASK-110
title: Pin managed sing-box installation to v1.13.21
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:52'
labels:
  - sing-box
  - setup
  - packaging
  - compatibility
milestone: m-7
dependencies:
  - TASK-98
references:
  - 'https://github.com/SagerNet/sing-box/releases/tag/v1.13.21'
priority: high
ordinal: 84000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Align xrat setup/install behavior with the runtime contract by installing the exact v1.13.21 sing-box release for supported platforms. Verify archive naming, checksums, executable discovery, upgrades, and coexistence with a user-supplied binary.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Managed installation downloads v1.13.21 from the official immutable release
- [ ] #2 Architecture/platform selection and checksums are verified before replacement
- [ ] #3 A user-configured external binary is not silently overwritten
- [ ] #4 Upgrade and status output distinguish managed and external binaries and show the detected version
- [ ] #5 Installer tests cover supported platform asset selection and checksum failure
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
