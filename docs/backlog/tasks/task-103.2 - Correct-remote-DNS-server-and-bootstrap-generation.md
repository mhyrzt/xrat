---
id: TASK-103.2
title: Correct remote DNS server and bootstrap generation
status: Done
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:51'
updated_date: '2026-09-19 21:09'
labels:
  - sing-box
  - dns
  - bootstrap
milestone: m-7
dependencies:
  - TASK-98
references:
  - 'https://sing-box.sagernet.org/configuration/dns/server/'
parent_task_id: TASK-103
priority: high
ordinal: 77000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Audit typed sing-box 1.13 server objects generated from Xrat DNS strings. Cover local, UDP, TCP, TLS, QUIC, HTTPS, and HTTP3 endpoints, default ports, IPv6 literals, paths, TLS SNI, and domain_resolver bootstrap dependencies.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every accepted scheme emits the correct type, server, port, path, and TLS fields
- [x] #2 IPv4, IPv6, and domain endpoints parse without ambiguity
- [x] #3 Domain endpoints reference a reachable local/bootstrap resolver without cycles
- [x] #4 Credentials, queries, fragments, and unsupported cleartext HTTP forms fail explicitly
- [x] #5 One fixture per supported server type passes sing-box v1.13.21 check
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Typed udp/tcp/tls/quic/https/h3/local servers with default ports, IPv6-safe parsing, path only for https/h3, TLS SNI, credentials/query/fragment rejection, and domain_resolver bootstrap to the local resolver. Native fixture covers every server type.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Remote and local DNS server generation verified with correct typing, bootstrap resolver, and native validation per server type.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
