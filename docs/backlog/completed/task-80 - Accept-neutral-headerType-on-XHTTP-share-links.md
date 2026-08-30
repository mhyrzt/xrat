---
id: TASK-80
title: Accept neutral headerType on XHTTP share links
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 18:58'
updated_date: '2026-08-25 19:03'
labels: []
dependencies: []
references:
  - 'https://github.com/XTLS/Xray-core/discussions/716'
modified_files:
  - src/xray/config/stream.rs
  - src/xray/config/generator/tests.rs
ordinal: 42000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Some imported VLESS XHTTP links include the generic legacy query parameter headerType=none. Runtime config generation currently rejects every XHTTP headerType, leaving otherwise valid configs unusable. Align handling with the official Xray share-link proposal without silently accepting meaningful unsupported XHTTP fields.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 XHTTP links with an absent, empty, or neutral headerType generate the same runtime transport settings
- [x] #2 Non-neutral headerType values on XHTTP remain rejected to avoid incomplete runtime config
- [x] #3 Regression tests cover accepted and rejected XHTTP headerType values
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Verify the failing records and official Xray share-link contract. 2. Consume only empty/none headerType values for XHTTP and reject non-neutral values. 3. Add focused generator regression tests and run formatting plus relevant test/CI checks.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Official Xray share-link proposal defines XHTTP path, host, mode, and extra; headerType is not an XHTTP field. Local database inspection found 341 XHTTP configs with headerType=none and no non-neutral values. Implemented narrow compatibility handling for absent/empty/none while retaining fail-closed rejection for other values. Validation: just fmt; cargo test -q --locked xhttp_; just ci (795 unit tests plus 1 integration test passed); patched cargo run -- test 2166aa08 reached ICMP/TCP/real-delay instead of failing config generation.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Accepted inert legacy headerType values on XHTTP share links without weakening validation for meaningful unsupported values. Added regression coverage and verified the exact reported config advances past generation; full CI passes.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
