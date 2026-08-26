---
id: TASK-91
title: Wire and modernize Xray inbound settings parsing
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 20:35'
updated_date: '2026-08-25 22:55'
labels:
  - bug
  - xray
  - parser
  - inbound
milestone: m-6
dependencies: []
references:
  - 'https://github.com/XTLS/Xray-core/tree/v26.3.27/infra/conf'
  - 'https://github.com/XTLS/Xray-core/tree/v26.7.28/infra/conf'
priority: high
ordinal: 53000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
InboundObject currently requires port and flattens protocol/settings as unvalidated JSON while the typed inbound settings structs have zero graph references and are stale. Connect protocol-aware parsing or deliberately replace the misleading dead schema, then align port optionality, users/clients aliases, NetworkList forms, VLESS flow/testseed, fallback type, and the prerelease TUN object.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Inbound protocol and settings are validated by the schema in strict mode
- [x] #2 Loose mode preserves unsupported inbound extensions without data loss
- [x] #3 Official users/clients compatibility aliases behave like Xray-core
- [x] #4 VLESS fallback and prerelease TUN fields match official names and shapes
- [x] #5 All supported inbound protocols have stable/prerelease fixture coverage
- [x] #6 Official inbound aliases such as mixed and dokodemo-door select the correct settings schema
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Dispatch inbound settings by protocol, make port optional, implement aliases/precedence, and preserve unknown loose-mode extensions.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verified: just fmt ci passed with Clippy warnings denied and 810 Rust tests plus the doctest. Native targeted configs passed Xray v26.3.27 and official v26.7.28. Cross-validated against tagged Xray-core source and Discussion 716. Documentation was updated in the reference and architecture guides.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added protocol-aware inbound parsing, optional ports, users/clients precedence, aliases, modern TUN and fallback fields, and loose preservation.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
