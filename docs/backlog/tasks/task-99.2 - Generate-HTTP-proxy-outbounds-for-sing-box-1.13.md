---
id: TASK-99.2
title: Generate HTTP proxy outbounds for sing-box 1.13
status: Done
assignee:
  - '@codex'
created_date: '2026-08-30 17:51'
updated_date: '2026-09-19 21:09'
labels:
  - sing-box
  - http
  - outbound
milestone: m-7
dependencies:
  - TASK-99.1
references:
  - 'https://sing-box.sagernet.org/configuration/outbound/http/'
parent_task_id: TASK-99
priority: medium
ordinal: 70000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add managed sing-box HTTP CONNECT upstream generation for plain HTTP and HTTPS proxy imports. Preserve credentials and enable TLS only when the source scheme/security requires it.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 HTTP proxy address, port, and optional credentials map exactly
- [x] #2 HTTPS proxy imports emit a valid enabled TLS block with correct server name behavior
- [x] #3 Plain HTTP imports do not accidentally enable TLS
- [x] #4 Malformed partial credentials or unsupported URL options fail before launch
- [x] #5 Representative HTTP and HTTPS fixtures pass sing-box v1.13.21 check
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
simple.rs HTTP path maps username/password, enables tls only for security=tls with server_name, and rejects credentials without username, host/path, and unknown link parameters. Native HTTP+HTTPS fixture passes.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
HTTP CONNECT upstream generation implemented with correct credential and HTTPS-only TLS handling.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
