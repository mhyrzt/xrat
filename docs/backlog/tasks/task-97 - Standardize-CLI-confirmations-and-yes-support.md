---
id: TASK-97
title: Standardize CLI confirmations and --yes support
status: To Do
assignee: []
created_date: '2026-08-26 11:17'
labels: []
dependencies: []
ordinal: 59000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Audit and standardize every terminal-interactive XRAT command. Current prompt surface: xrat setup (dependency, daemon, and lingering choices), xrat delete subscription, xrat purge, and xrat logs clear. These already share y/yes parsing and expose --yes. xrat delete config --hard is an irreversible gap: it deletes immediately and has no --yes flag. install.sh delegates interaction to xrat setup and already forwards -y. TUI inline key confirmations are out of scope because they are not CLI/non-interactive command prompts.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Every destructive CLI path prompts with a safe default unless --yes is supplied, including delete config --hard
- [ ] #2 Every CLI command that may prompt exposes a documented --yes flag suitable for non-interactive sessions
- [ ] #3 Confirmation input accepts y and yes case-insensitively, rejects other input safely, and never blocks when stdin is not a terminal
- [ ] #4 Prompts use one color-aware presentation with clear action, target or affected count, explicit y/N choices, and consistent abort/success output
- [ ] #5 CLI parsing and command tests cover interactive acceptance, rejection/default behavior, non-TTY behavior, and --yes bypass for each command family
- [ ] #6 CLI documentation lists confirmation behavior and --yes examples for setup, delete config --hard, delete subscription, purge, and logs clear
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
