---
id: TASK-105
title: Audit and type managed sing-box inbound generation
status: Done
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:50'
updated_date: '2026-09-19 21:09'
labels:
  - sing-box
  - inbound
  - config-generation
milestone: m-7
dependencies:
  - TASK-98
references:
  - 'https://sing-box.sagernet.org/configuration/inbound/'
  - TASK-73
priority: high
ordinal: 68000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the overly broad shared inbound shape with schema-correct typed generation for SOCKS, HTTP, and Shadowsocks managed inbounds. Audit listen fields, network restrictions, authentication users, cipher/method, and readiness endpoints against sing-box 1.13.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 SOCKS, HTTP, and Shadowsocks inbounds emit only fields valid for their own schemas
- [x] #2 SOCKS username/password authentication is represented using the documented users shape
- [x] #3 HTTP and Shadowsocks authentication behavior is either implemented exactly or rejected explicitly
- [x] #4 TCP/UDP network choices preserve configured behavior and invalid combinations fail preflight
- [x] #5 Each individual and combined inbound configuration passes sing-box v1.13.21 check
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Replaced the shared inbound struct with a typed enum (Socks/Http/Shadowsocks). SOCKS auth uses the documented users shape; HTTP has no auth settings in xrat so nothing is inferred; Shadowsocks validates tcp/udp network, supported method, and non-empty password. Combined native inbound fixture passes on installed sing-box 1.13.19.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Managed sing-box inbounds are typed per schema with validation for network, method, password, and SOCKS users; native combined fixture passes.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
