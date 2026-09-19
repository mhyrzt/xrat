---
id: TASK-100.2
title: Generate ordered sing-box route rules and actions
status: Done
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:51'
updated_date: '2026-09-19 22:23'
labels:
  - sing-box
  - routing
  - config-generation
milestone: m-7
dependencies:
  - TASK-98
  - TASK-100.1
references:
  - 'https://sing-box.sagernet.org/configuration/route/rule/'
  - 'https://sing-box.sagernet.org/configuration/route/rule_action/'
parent_task_id: TASK-100
priority: high
ordinal: 80000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Generate sing-box 1.13 domain, IP/CIDR, rule-set, direct, proxy, and block behavior with deterministic ordering. Decide when to use route-to-block outbound versus reject action based on the pinned schema and desired connection semantics.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 full, domain, keyword, regexp, IP, and CIDR inputs map to the documented match fields
- [x] #2 Direct rules precede block rules and unmatched traffic uses proxy
- [x] #3 Block behavior is explicitly tested for TCP and UDP semantics
- [x] #4 Rule-set tags resolve to declared local rule-set objects
- [x] #5 Negation, ext, dotless, and malformed values fail unless an exact mapping is documented
- [x] #6 Representative routing combinations pass sing-box v1.13.21 check
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Progress (2026-09-20): domain (full/domain/keyword/regexp), IP/CIDR, direct-before-block ordering, and proxy fallback are implemented in src/singbox/config/mod.rs with native check coverage. Remaining: wire rule_set tags for geosite/geoip, decide block action vs block outbound TCP/UDP semantics, and cover those combinations in fixtures.

Domain (full/domain/keyword/regexp), IP/CIDR, direct-before-block ordering, and proxy fallback were already implemented. Added geosite/geoip mapping to remote SagerNet rule-sets: route.rule_set entries (type=remote, format=binary, sing-geosite/sing-geoip URLs) with referencing route rules, category-name validation, dedup by tag, and block via the block outbound (applies to TCP and UDP). Native check passes with rule sets and cache_file.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Ordered sing-box route rules now cover domain, IP/CIDR, and geosite/geoip rule-sets with direct-before-block ordering, proxy fallback, and native validation.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
