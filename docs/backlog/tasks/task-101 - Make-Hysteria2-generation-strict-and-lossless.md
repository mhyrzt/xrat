---
id: TASK-101
title: Make Hysteria2 generation strict and lossless
status: Done
assignee:
  - '@codex'
created_date: '2026-08-30 15:36'
updated_date: '2026-09-18 16:07'
labels:
  - sing-box
  - hysteria2
  - config-generation
  - bug
milestone: m-7
dependencies:
  - TASK-98
references:
  - TASK-73
  - 'https://sing-box.sagernet.org/configuration/outbound/hysteria2/'
  - 'https://github.com/yarikov/kvn-tui'
priority: high
ordinal: 63000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Correct the Hysteria2 outbound mapping so invalid or unsupported share-link intent is never silently dropped. The current builder defaults a missing password to an empty string, ignores unsupported obfs values, and silently omits malformed upmbps/downmbps values. Preserve every supported official Hysteria2 field and fail before launch for values Xrat cannot represent.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Missing or empty Hysteria2 authentication is rejected before config generation
- [x] #2 Unsupported obfs types and malformed bandwidth values produce actionable errors instead of being ignored
- [x] #3 Supported insecure, SNI, ALPN, Salamander obfuscation, password, and bandwidth values serialize exactly to the documented sing-box shape
- [x] #4 Generated probe and managed-runtime Hysteria2 configs pass the native validator for every supported case
- [x] #5 Regression tests cover raw imported links and database-restored extension maps
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Treat tagged v1.13.21 Hysteria2OutboundOptions and TLS options as the field authority. 2. Preserve password, network, TLS, Salamander, and bandwidth exactly; reject missing authentication, invalid values, non-scalar restored extensions, and unsupported query keys. 3. Add raw-link and restored-extension regression tests plus native-validator coverage. 4. Update the support matrix and validate focused tests before finalizing.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verified against tagged v1.13.21 Hysteria2OutboundOptions and outbound TLS options. The builder now preserves password, tcp/udp network, SNI, insecure, ALPN, Salamander, and integer bandwidth, and rejects empty authentication, unsupported/unknown URI fields, malformed bandwidth, invalid restored extension types, and lossy obfs input. Validation passed: singbox::config::tests, native_singbox_validator_accepts_generated_dns_config (installed 1.13.19), cargo clippy --all-targets -- -D warnings, just fmt, just mdbook-build, and git diff --check.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Made Hysteria2 sing-box generation strict and lossless for the supported v1.13 shape, with raw-link and restored-extension regression coverage. Unsupported 1.14 fields and unrepresentable inputs now fail before configuration generation.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
