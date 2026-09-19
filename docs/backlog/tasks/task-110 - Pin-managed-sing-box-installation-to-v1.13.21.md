---
id: TASK-110
title: Pin managed sing-box installation to v1.13.21
status: Done
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:52'
updated_date: '2026-09-19 22:06'
labels:
  - sing-box
  - setup
  - packaging
  - compatibility
milestone: m-7
dependencies:
  - TASK-98
references:
  - 'https://github.com/SagerNet/sing-box/releases/tag/v1.13.21'
priority: high
ordinal: 84000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Align xrat setup/install behavior with the runtime contract by installing the exact v1.13.21 sing-box release for supported platforms. Verify archive naming, checksums, executable discovery, upgrades, and coexistence with a user-supplied binary.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Managed installation downloads v1.13.21 from the official immutable release
- [x] #2 Architecture/platform selection and checksums are verified before replacement
- [x] #3 A user-configured external binary is not silently overwritten
- [x] #4 Upgrade and status output distinguish managed and external binaries and show the detected version
- [x] #5 Installer tests cover supported platform asset selection and checksum failure
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Decision (2026-09-20): pin the managed sing-box install to the immutable v1.13.21 release (hardcoded URL + published SHA-256) so the managed install matches the TASK-102/TASK-109 conformance target. The runtime gate keeps accepting any stable >=1.13.0,<1.14.0 binary, so user-supplied 1.13.x works. Installing latest 1.13.x at setup time was considered and rejected because it needs a GitHub API lookup, drifts within 1.13 with no xrat change, and has no fixed checksum at build time. Keep the version in a single constant so a future pin bump is one line.

Added PINNED_SINGBOX_VERSION=1.13.21 and default_version(kind): sing-box setup/install now resolves the immutable tags/v1.13.21 release while Xray/V2Ray still track latest stable. Probe detail labels sing-box as pinned. xrat install sing-box reports the pinned channel. Existing managed/external handling and checksum verification are unchanged. Tests: pins_the_managed_sing_box_release plus existing asset-selection and checksum-failure coverage.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Managed sing-box installation is pinned to the immutable v1.13.21 release; the runtime still accepts any stable 1.13.x. Platform asset selection, checksum verification, and managed/external separation are preserved and tested.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
