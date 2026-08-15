---
id: TASK-60
title: Harden automatic rotation health and runtime handoff
status: Done
assignee: []
created_date: '2026-08-14 23:26'
updated_date: '2026-08-15 00:23'
labels:
  - rotation
  - reliability
  - bugfix
dependencies: []
priority: high
ordinal: 20000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make rotation detect real data-plane failures without flapping, validate replacement configs before disruption, and recover the previous runtime when a handoff fails. Persist rotate enable/disable in config and align CLI, daemon, TUI, and documentation semantics.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Process exit and inbound loss trigger immediate recovery while proxied HTTP failures require the configured consecutive-failure threshold
- [x] #2 Health probes use the active runtime proxy, run asynchronously, discard stale results, and expose health state and reason codes
- [x] #3 Automatic and unpinned rotation use only fresh eligible test results; an explicit config ID may bypass health and cooldown but must be enabled, non-active, and valid
- [x] #4 Replacement configs pass native engine validation before the active runtime stops and failed handoffs roll back to the previous runtime
- [x] #5 Rotate enable and disable persist atomically and timer failures wait for the normal interval before retrying
- [x] #6 Tests and docs cover thresholds, candidate policy, idle behavior, rollback, and persistent controls
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add the rotation health threshold setting and persisted enable/disable updates. 2. Add layered process, socket, and proxied-HTTP health state with asynchronous stale-safe probes. 3. Tighten candidate qualification and timer retry behavior. 4. Add native preflight and transactional handoff rollback. 5. Add regression tests and update rotation documentation.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented immediate process/inbound recovery, thresholded asynchronous proxied HTTP health checks with stale-result rejection, fresh candidate testing, explicit-target policy, native Xray/V2Ray/sing-box preflight, rollback on failed handoff, persistent rotate enable/disable, normal-interval retry, status diagnostics, and TUI settings integration (Failure threshold). Updated rotation, CLI, daemon, architecture, configuration, and settings documentation. Validation: just fmt ci passed (737 tests; clippy with -D warnings; Rust/Markdown/SQL formatting).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Hardened rotation health detection and handoff safety, persisted scheduler controls, exposed health diagnostics, and integrated the new threshold setting throughout config defaults, validation, TUI help/labels, examples, and docs.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
