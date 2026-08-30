---
id: TASK-82
title: Audit Xray parsing against stable and prerelease
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 20:24'
updated_date: '2026-08-25 20:38'
labels:
  - audit
  - xray
  - parser
  - config-generation
milestone: m-6
dependencies: []
references:
  - 'https://github.com/XTLS/Xray-core/releases'
  - 'https://github.com/XTLS/Xray-core/discussions/716'
priority: high
ordinal: 44000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Inventory the full src/xray/parsing JSON schema parser plus XRAT share-link parsing and runtime-generation boundaries. Compare accepted fields, aliases, enum values, strict/loose behavior, and emitted JSON against current official stable and prerelease Xray-core documentation/source; reproduce each verified mismatch and create a separate milestone task for every actionable defect. This task is audit-only and does not implement fixes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Stable and prerelease Xray-core versions and authoritative source references are recorded
- [x] #2 All src/xray/parsing schema objects plus supported share-link/runtime boundaries are mapped to official Xray fields
- [x] #3 Every suspected mismatch is confirmed by official source or native validation before task creation
- [x] #4 Each verified defect has a scoped Backlog task in the Xray Schema Compatibility milestone
- [x] #5 The audit records checked areas with no mismatch and concludes with no unclassified findings
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Pin official stable and prerelease Xray-core versions and collect their config schemas. 2. Inventory src/xray/parsing by core objects, protocol settings, transports, and security; then inspect share-link/runtime boundaries. 3. Compare aliases, required fields, enum values, and strict/loose behavior and reproduce mismatches using tagged sources or native validation. 4. Search Backlog for duplicates and create one milestone task per verified defect. 5. Record negative coverage, finalize the audit, and leave implementation for later tasks.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Audit baseline:
- Stable: Xray-core v26.3.27, published 2026-03-27; locally installed xray reports 26.3.27.
- Prerelease: Xray-core v26.7.28, published 2026-07-28.
- Authoritative inputs: tagged infra/conf source for both versions, tagged release metadata, current Xray transport documentation, and official VMessAEAD/VLESS share-link proposal Discussion 716.
- Scope mapped: every file under src/xray/parsing/core, protocols, shared, transports, and security; src/config protocol link parsers; src/xray/config outbound, stream, extension, and serialization boundaries.
- Confirmation method: exact Go json tags, loader aliases, Build requiredness/precedence, and stable-to-prerelease source diffs. No speculative findings were converted into tasks.

Classified findings:
- TASK-83 root/core objects and acronym-sensitive fields.
- TASK-84 DNS and routing.
- TASK-85 stream selectors, aliases, address/port, and root transport shape.
- TASK-86 WebSocket, gRPC, and HTTPUpgrade.
- TASK-87 stable/prerelease mKCP divergence.
- TASK-88 TLS and REALITY.
- TASK-89 sockopt and custom socket options.
- TASK-90 outbound protocols, aliases, direct and servers/vnext layouts.
- TASK-91 inbound protocol dispatch and stale disconnected settings schemas.
- TASK-92 recursive strictness and lossless loose parsing.
- TASK-93 current VMess/VLESS link fields.
- TASK-94 URL-form VMess AEAD links.

Checked without a separate mismatch:
- Log levels and address masking, version, stats, reverse, basic blackhole/loopback payloads, sniffing fields, query/domain strategy enums, and port scalar/range representation align for the audited surface.
- Raw, XHTTP, Hysteria, and finalmask full-JSON settings use flattened maps and therefore preserve their evolving nested payloads; selector and link-boundary defects are tracked separately.
- Existing XHTTP extra handling now follows the official JSON-extra contract from TASK-75.
- No remaining finding is unclassified; version-only removals are assigned to the relevant compatibility task rather than treated as universal schema support.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed an audit-only comparison of the full Xray parser and share-link/runtime boundaries against stable v26.3.27 and prerelease v26.7.28. Created twelve scoped fix tasks in milestone m-6 and recorded aligned areas; no parser finding remains unclassified.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
