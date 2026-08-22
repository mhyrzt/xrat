---
id: TASK-70
title: Expose actionable subscription update HTTP errors
status: To Do
assignee: []
created_date: '2026-08-18 11:02'
labels:
  - bug
  - subscription
  - cli
dependencies: []
references:
  - src/app/error.rs
  - src/app/input/source.rs
priority: medium
ordinal: 30000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The subscription updater currently collapses all reqwest transport failures and non-success HTTP responses into the generic message HTTP request failed. This makes xrat update unable to distinguish DNS failure, connection timeout, TLS failure, or an HTTP status such as 401/404. Preserve the per-subscription failure summary while exposing a useful, sanitized cause without leaking subscription credentials or full sensitive URLs. A recent feyvpn update reproduced a TCP timeout before any HTTP response.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Subscription update output distinguishes transport failures from non-success HTTP responses.
- [ ] #2 Transport diagnostics identify useful causes such as DNS, timeout, connection, or TLS failure when available.
- [ ] #3 HTTP failures include the response status and a safe response detail when available.
- [ ] #4 Sensitive subscription tokens and credentials are not included in diagnostics.
- [ ] #5 Tests cover representative transport and HTTP-status failures while preserving existing update summary behavior.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
