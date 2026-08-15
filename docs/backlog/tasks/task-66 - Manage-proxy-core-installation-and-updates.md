---
id: TASK-66
title: Manage proxy-core installation and updates
status: Done
assignee:
  - '@codex'
created_date: '2026-08-15 14:17'
updated_date: '2026-08-15 14:38'
labels:
  - feature
  - setup
  - runtime
dependencies: []
references:
  - 'https://github.com/XTLS/Xray-install'
  - 'https://sing-box.app/install.sh'
  - 'https://github.com/v2fly/fhs-install-v2ray'
priority: high
ordinal: 26000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make xrat setup detect, install, and update Xray, sing-box, and V2Ray from verified official release artifacts. Managed copies remain user-local and do not overwrite package-managed binaries.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Interactive setup reports missing and outdated Xray, sing-box, and V2Ray and offers the planned actions
- [x] #2 Managed installations use official stable release artifacts, verify SHA-256, and remain usable after a failed update
- [x] #3 Managed cores are available through user-local CLI links and xrat persists the managed runtime paths
- [x] #4 Setup yes installs missing Xray and sing-box, skips absent V2Ray, and updates installed outdated cores
- [x] #5 Setup check remains read-only and reports update_available in table and JSON output
- [x] #6 Piped install.sh can use terminal prompts and has a clear non-interactive fallback
- [x] #7 Linux and macOS x86_64 and arm64 mappings, external adoption, collisions, and failures are tested
- [x] #8 User-facing setup and installation documentation is updated
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add release discovery, version probing, verified archive extraction, and atomic user-local core installation. 2. Integrate dependency decisions and reporting into setup. 3. Make install.sh preserve interactive prompts through /dev/tty. 4. Add regression tests and documentation, then run just fmt ci.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented typed release discovery, SHA-256 verification, staged archive extraction, rollback-safe core replacement, external adoption, user-local CLI links, setup prompt policies, update_available reporting, managed asset paths, install.sh TTY handoff, and docs. Focused setup tests: 24 passed. install.sh passes bash syntax validation.

Final validation passed: just fmt ci; 766 library tests and 1 binary test passed, clippy is warning-free, documentation and SQL formatters are clean, install.sh passes bash -n, and git diff --check is clean. No live proxy core was installed during testing.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added verified user-local Xray, sing-box, and V2Ray installation and upgrades to xrat setup. Setup now reports missing/outdated cores, applies interactive and unattended policies, adopts external cores without overwriting them, persists managed paths, exposes safe CLI links, and reports update_available in check output. install.sh now preserves prompts through /dev/tty. Verified with the complete project CI gate and offline archive/config regression tests.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
