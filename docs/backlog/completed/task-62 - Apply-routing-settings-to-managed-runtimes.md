---
id: TASK-62
title: Apply routing settings to managed runtimes
status: Done
assignee: []
created_date: '2026-08-15 11:20'
updated_date: '2026-08-15 11:44'
labels: []
dependencies: []
references:
  - 'https://xtls.github.io/en/config/routing.html'
  - 'https://xtls.github.io/en/config/outbounds/blackhole.html'
priority: high
ordinal: 22000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make existing routing.domain_strategy, routing.direct, and routing.block settings affect generated managed runtime configurations instead of only the PAC subset. Preserve safe runtime replacement and engine-specific validation.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Managed Xray and V2Ray sessions emit ordered Direct and Block domain, IP, Geosite, and GeoIP routing rules
- [x] #2 Managed sing-box sessions support translatable domain and IP rules and reject unsupported rule forms clearly
- [x] #3 Direct rules take precedence over Block rules and unmatched traffic uses the selected proxy
- [x] #4 Stats API routing is merged without overwriting user rules
- [x] #5 Probes and parser previews remain proxy-only
- [x] #6 Configuration validation, TUI help, documentation, and regression tests describe actual behavior
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add typed engine routing models and generators. 2. Pass routing through managed runtime launch while excluding probes. 3. Add validation and preserve stats/network tuning behavior. 4. Update TUI/docs and verify with unit tests plus native engine validators.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented managed-runtime routing generation for Xray/V2Ray and sing-box, preserved stats-rule precedence, kept probes proxy-only, added native preflight safety for manual active-session replacement, config validation, TUI help, documentation, and regression tests. sing-box geosite/GeoIP remains explicitly rejected and is tracked by TASK-63.

Validation: just fmt ci passed, including rustfmt, Prettier, SQL formatting, clippy with warnings denied, and all 747 tests. Representative generated Xray and sing-box routing documents were also checked with the installed native validators during planning.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Applied routing settings to managed Xray/V2Ray and sing-box sessions with deterministic Direct-before-Block precedence and proxy fallback. Preserved stats routing, network tuning, and proxy-only probes; added clear sing-box limitations, validation, replacement preflight safety, TUI help, docs, and regression coverage. Full sing-box GeoIP/Geosite rule-set support remains tracked by TASK-63.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
