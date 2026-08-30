---
id: TASK-90
title: Correct Xray outbound protocol settings
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 20:35'
updated_date: '2026-08-25 22:55'
labels:
  - bug
  - xray
  - parser
  - outbound
milestone: m-6
dependencies: []
references:
  - 'https://github.com/XTLS/Xray-core/tree/v26.3.27/infra/conf'
  - 'https://github.com/XTLS/Xray-core/tree/v26.7.28/infra/conf'
priority: high
ordinal: 52000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Align typed outbound settings with official stable and prerelease protocol structs. The parser cannot read xrat-generated servers/vnext layouts for multiple protocols and its direct forms use VLESS/VMess uuid instead of id. Other confirmed gaps include mux xudpProxyUDP443 spelling, Freedom targetStrategy/noise/finalRules, prerelease DNS rewrite fields/rules, and outdated WireGuard options.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 VLESS and VMess direct and vnext forms use official id fields and parse generated xrat configs
- [x] #2 Mux and WireGuard acronym-sensitive keys match Xray-core
- [x] #3 Freedom stable fields and prerelease finalRules are represented
- [x] #4 DNS outbound legacy and prerelease rewrite/rule forms are represented with defined precedence
- [x] #5 Protocol fixtures validate and round-trip on both audited versions
- [x] #6 HTTP, SOCKS, Shadowsocks, Trojan, VLESS, and VMess accept every direct and servers/vnext form supported by the audited Xray versions
- [x] #7 Official outbound protocol aliases block and direct normalize to blackhole and freedom behavior
- [x] #8 Fields removed between stable and prerelease have explicit compatibility behavior instead of silent loss
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Support direct and servers/vnext outbound forms, protocol aliases, DNS/Freedom version fields, and WireGuard/Mux corrections.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verified: just fmt ci passed with Clippy warnings denied and 810 Rust tests plus the doctest. Native targeted configs passed Xray v26.3.27 and official v26.7.28. Cross-validated against tagged Xray-core source and Discussion 716. Documentation was updated in the reference and architecture guides.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Corrected outbound direct and servers/vnext forms, aliases, precedence helpers, Freedom and DNS variants, and Mux and WireGuard keys.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
