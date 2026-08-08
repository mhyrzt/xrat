---
id: TASK-51
title: Add configurable real-delay HTTP status acceptance
status: Done
assignee:
  - '@codex'
created_date: '2026-08-08 11:55'
updated_date: '2026-08-08 12:09'
labels:
  - feature
  - testing
dependencies: []
priority: medium
ordinal: 9000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Allow real-delay tests to accept multiple exact HTTP status codes and inclusive ranges, with explicit redirect-following behavior. Preserve 2xx acceptance for existing configs. Related to TASK-34 but does not depend on its broader HTTP-client refactor.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Existing configs continue to accept final HTTP statuses 200-299 and follow redirects
- [x] #2 Users can configure exact accepted status codes and inclusive status ranges with replacement semantics
- [x] #3 Users can disable redirect following to accept an initial 3xx response
- [x] #4 Redirect chains are limited to 10 hops and loops fail with a clear reason
- [x] #5 Invalid status codes, ranges, and empty configured acceptance sets are rejected
- [x] #6 Tests and user-facing configuration documentation cover the new behavior
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Extend real-delay config types, defaults, resolution, and validation. 2. Add a reusable status matcher and configurable reqwest redirect policy. 3. Add parser, validation, status, redirect, and loop tests using a local HTTP server. 4. Update generated/example config and user documentation. 5. Run focused tests and just fmt ci.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented optional exact status codes, inclusive ranges, replacement semantics, configurable redirect following, a fixed 10-hop limit, actionable validation, local HTTP redirect tests, and user documentation. Verification: focused config, validation, settings, and real-delay tests passed; just fmt ci passed; 679 full tests passed; testdata/config.example.toml validates.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Real-delay tests now support multiple exact HTTP statuses and inclusive ranges, can inspect initial redirects or terminal responses, and fail redirect loops after 10 hops. Existing configs retain final 200-299 behavior. Documentation and validation were updated, with the full CI gate passing.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
