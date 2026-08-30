---
id: TASK-109
title: Gate release on sing-box v1.13.21 conformance
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:52'
labels:
  - sing-box
  - ci
  - release
  - testing
milestone: m-7
dependencies:
  - TASK-102
references:
  - 'https://github.com/SagerNet/sing-box/releases/tag/v1.13.21'
priority: high
ordinal: 83000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a deterministic release/CI gate that obtains the pinned official sing-box v1.13.21 binary and runs the complete generated-config conformance matrix. Prevent release artifacts from shipping when supported output is rejected or the validator was silently skipped.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 CI downloads or caches the official v1.13.21 binary with checksum verification
- [ ] #2 The full conformance matrix runs on every relevant Rust/config-generation change
- [ ] #3 Validator absence, download failure, or skipped fixtures fail the release gate
- [ ] #4 Failures retain the fixture name, generated JSON, command output, and sing-box version
- [ ] #5 Release documentation records the validated sing-box version
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
