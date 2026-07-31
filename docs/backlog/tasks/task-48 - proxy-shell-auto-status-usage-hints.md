---
id: TASK-48
title: 'proxy shell: auto status + usage hints'
status: To Do
assignee: []
created_date: '2026-07-31 21:08'
labels:
  - cli
  - proxy-shell
dependencies: []
ordinal: 6000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
xrat proxy shell enable/disable/toggle should automatically print the current proxy shell status after running, and print how to use the proxy with respect to the user's shell (as a comment or inside --help).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 enable/disable/toggle print xrat proxy shell status after execution
- [ ] #2 Usage instructions are printed as a shell comment or shown in --help
- [ ] #3 Usage hint reflects user shell (bash/zsh/fish/etc.)
- [ ] #4 Tests cover status printing and usage hint generation
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
