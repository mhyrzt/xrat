---
id: TASK-101
title: Make Hysteria2 generation strict and lossless
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 15:36'
labels:
  - sing-box
  - hysteria2
  - config-generation
  - bug
milestone: m-7
dependencies:
  - TASK-98
references:
  - TASK-73
  - 'https://sing-box.sagernet.org/configuration/outbound/hysteria2/'
  - 'https://github.com/yarikov/kvn-tui'
priority: high
ordinal: 63000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Correct the Hysteria2 outbound mapping so invalid or unsupported share-link intent is never silently dropped. The current builder defaults a missing password to an empty string, ignores unsupported obfs values, and silently omits malformed upmbps/downmbps values. Preserve every supported official Hysteria2 field and fail before launch for values Xrat cannot represent.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Missing or empty Hysteria2 authentication is rejected before config generation
- [ ] #2 Unsupported obfs types and malformed bandwidth values produce actionable errors instead of being ignored
- [ ] #3 Supported insecure, SNI, ALPN, Salamander obfuscation, password, and bandwidth values serialize exactly to the documented sing-box shape
- [ ] #4 Generated probe and managed-runtime Hysteria2 configs pass the native validator for every supported case
- [ ] #5 Regression tests cover raw imported links and database-restored extension maps
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
