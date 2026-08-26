---
id: TASK-75
title: Harden Xray link parameters and support XHTTP extras
status: Done
assignee:
  - '@codex'
created_date: '2026-08-23 11:08'
updated_date: '2026-08-23 11:17'
labels:
  - bug
  - xray
  - xhttp
  - config-generation
dependencies: []
references:
  - 'https://github.com/mhyrzt/xrat/issues/2'
  - >-
    https://github.com/XTLS/Xray-core/discussions/4113#discussioncomment-17385775
documentation:
  - docs/src/05-reference/protocols.md
  - docs/src/06-architecture/config-generation.md
modified_files:
  - src/xray/config/extensions.rs
  - src/xray/config/mod.rs
  - src/xray/config/outbound.rs
  - src/xray/config/stream.rs
  - src/xray/config/generator/tests.rs
  - src/db/database/tests/import_cases/upsert.rs
  - docs/src/05-reference/protocols.md
  - docs/src/06-architecture/config-generation.md
priority: high
ordinal: 37000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Resolve GitHub issue #2 by generating XHTTP extra settings correctly and replacing the detached global Xray link-parameter allowlist with typed consumption tracking so accepted parameters cannot be silently discarded.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 VLESS XHTTP links map official extra JSON and supported flat fields into xhttpSettings.extra with documented precedence
- [x] #2 xPaddingBytes camelCase, x_padding_bytes, and the reported x_padding bytes alias generate the canonical Xray field
- [x] #3 All Xray protocol and transport parameters are validated by actual typed consumption rather than a detached allowlist
- [x] #4 Unknown, malformed, repeated, conflicting, or wrong-context parameters fail with actionable errors instead of being dropped
- [x] #5 Regression, persistence, and optional native Xray validator tests cover supported and rejected behavior
- [x] #6 Protocol and config-generation documentation explains XHTTP extra compatibility and future-field guidance
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Introduce typed extension consumption shared by Xray outbound and stream generation and remove the global allowlist. 2. Build schema-aware XHTTP extra objects from official JSON plus current flat fields and explicit padding aliases. 3. Add end-to-end, failure, persistence, and native-validator regression tests. 4. Update documentation, run just fmt ci, and finalize the task without posting to GitHub issue #2.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented a shared typed ExtensionResolver; Xray builders now consume parameters in context and reject leftovers. XHTTP merges padding aliases, official extra JSON, and canonical flat fields while preserving unknown nested future fields. Validation passed: just fmt, just ci (788 tests total), focused Xray tests, persistence round-trip, and installed native Xray XHTTP validation.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Fixed issue #2 by emitting XHTTP extended parameters under xhttpSettings.extra and replacing the detached allowlist with typed consumption tracking. Added compatibility aliases, deterministic precedence, actionable validation failures, persistence/end-to-end/native tests, and user/architecture documentation.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
