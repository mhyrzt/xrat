---
id: TASK-47
title: 'proxy shell: select protocol on enable'
status: To Do
assignee: []
created_date: '2026-07-31 21:08'
labels:
  - cli
  - proxy-shell
dependencies: []
ordinal: 5000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
xrat proxy shell enable should accept a trailing flag or positional arg indicating which proxy protocol to use: http, socks5, or socks5h. Applies to the enable subcommand (last positional/flag).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 xrat proxy shell enable supports selecting http, socks5, or socks5h protocol
- [ ] #2 Protocol chosen via a last positional arg or a flag on the enable subcommand
- [ ] #3 Default behavior preserved when protocol not specified
- [ ] #4 CLI parser tests cover each protocol value and the default
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
