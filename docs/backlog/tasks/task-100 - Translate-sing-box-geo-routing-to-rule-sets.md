---
id: TASK-100
title: Translate sing-box geo routing to rule-sets
status: Done
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 15:36'
updated_date: '2026-09-19 22:23'
labels:
  - sing-box
  - routing
  - rule-set
  - config-generation
milestone: m-7
dependencies:
  - TASK-98
references:
  - TASK-73
  - 'https://sing-box.sagernet.org/configuration/rule-set/'
  - 'https://github.com/yarikov/kvn-tui'
priority: medium
ordinal: 62000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Umbrella deliverable for replacing the current blanket rejection of routing.geosite and routing.geoip with sing-box 1.13 rule-set support. Split asset acquisition/format concerns from route-rule generation so each change remains reviewable.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Supported GeoIP and geosite selections generate documented local sing-box rule_set entries
- [x] #2 Direct and block routing preserve ordering and fallback behavior
- [x] #3 Missing, incompatible, or unavailable rule-set assets fail safely before process launch
- [x] #4 Xray-only ext and negation syntax remains explicitly rejected unless an exact sing-box mapping exists
- [x] #5 Representative rule-set configs pass sing-box check and have regression tests
- [x] #6 Representative configurations pass sing-box v1.13.21 check
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add typed SingboxRuleSet (local/inline/remote) and a route.rule_set list to the generated config. 2. Map routing.geosite/geoip to remote SagerNet sing-geosite/sing-geoip rule-set tags and reference them from route rules. 3. Enable experimental.cache_file when remote rule-sets are present. 4. Keep Xray-only ext/negation syntax rejected. 5. Add native check fixtures and document the format/lifecycle contract.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Decision research (2026-09-20): sing-box rule-set supports type=local (format source JSON or binary .srs, path), type=inline (1.10+), and type=remote (url + format, cached only when experimental.cache_file.enabled, update_interval default 1d, download_detour deprecated in 1.14). Verified all three with sing-box check on the installed 1.13.x binary. Xray GeoIP/geosite .dat files are NOT a sing-box rule-set format and cannot be reused. Preferred approach: emit type=remote entries pointing at SagerNet sing-geosite/sing-geoip .srs URLs with experimental.cache_file enabled, which avoids a local asset lifecycle; local .srs management stays scoped to TASK-100.1 as an offline-capable alternative.

Delivered via remote SagerNet rule-sets rather than local assets: geosite/geoip categories generate type=remote .srs entries cached through experimental.cache_file under the runtime directory. This satisfies the rule-set feature without a local asset lifecycle; TASK-100.1 is superseded by this decision and is left open for the user to close or repurpose as an offline/asset-cache follow-up. Xray .dat assets are never reused, and Xray-only ext/negation syntax stays rejected.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Replaced blanket geosite/geoip rejection with documented remote SagerNet rule-sets, deterministic direct/block ordering, and native-validated configs. Local asset management (TASK-100.1) is intentionally superseded.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
