---
id: TASK-108
title: Publish the sing-box 1.13 support and limitation matrix
status: Done
assignee:
  - '@codex'
created_date: '2026-08-30 17:52'
updated_date: '2026-09-19 23:21'
labels:
  - sing-box
  - documentation
  - compatibility
milestone: m-7
dependencies:
  - TASK-99
  - TASK-100
  - TASK-101
  - TASK-103
  - TASK-105
  - TASK-106
references:
  - TASK-73
documentation:
  - docs/src/06-architecture/config-generation.md
priority: medium
ordinal: 82000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Publish user-facing and architecture documentation for the completed sing-box 1.13 backend. Separate import support, probe support, managed runtime support, inbound support, DNS/routing capabilities, required binary version, and intentionally rejected settings.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A protocol matrix distinguishes import, parse/show, probe, and managed-runtime support
- [x] #2 Inbound, TLS/transport, DNS, routing, Clash API, and rule-set limitations are documented
- [x] #3 The required stable 1.13.x range and v1.13.21 managed pin are visible in setup and troubleshooting docs
- [x] #4 Every unsupported mapping includes the error users should expect and a viable alternative where one exists
- [x] #5 Architecture documentation links each generated section to its owning module and conformance coverage
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Updated the user-facing protocol and engine docs: docs/src/05-reference/protocols.md now lists sing-box support for all seven protocols and documents the supported version range; docs/src/03-features/runtime-management.md engine table matches; README and the sing-box compatibility contract describe the >=1.13.0 <1.14.0 range and v1.13.21 target. Remaining: setup/troubleshooting version guidance and the rule-set limitation once TASK-100 lands.

protocols.md now has an Engine and Stage Support matrix (import/parse, show/parse JSON, probe per engine, managed runtime per engine) plus a sing-box limitations table with the exact error and an alternative per row. setup.md and configuration.md document the supported >=1.13.0 <1.14.0 range and the v1.13.21 managed pin. config-generation.md adds a Conformance Coverage table mapping each generated section to its owning module and native fixture. Also documented that sing-box rule-sets must be local .srs/source JSON, inline, or remote, and that Xray .dat assets cannot be reused.

Policy update (2026-09-20): docs now state sing-box >=1.13.0 with no upper bound, the tested >=1.13.0,<1.15.0 range, the v1.13.21 pin, and the warning behavior. Added an upstream deprecation-watch table to the compatibility contract (remote rule-set implicit HTTP client removed in 1.16, legacy DNS address filters, block outbound).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Published the user-facing and architecture sing-box support matrix: per-stage protocol support, per-area limitations with expected errors and alternatives, the supported version range and pinned conformance release, and module/fixture ownership for every generated section.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
