---
id: TASK-72.1
title: Apply DNS settings to Xray probe testing
status: Done
assignee:
  - '@mhyrzt'
created_date: '2026-08-23 01:09'
updated_date: '2026-08-23 01:14'
labels: []
dependencies: []
parent_task_id: TASK-72
priority: medium
ordinal: 34000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Use the configured [dns] settings when generating Xray probe configurations for testing. Real-delay requests currently run through a probe config built from runtime tuning only, so custom DNS is ignored. Apply DNS to the shared proxy-based test configuration path without adding managed-runtime routing behavior.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Real-delay probe configs include the configured Xray DNS object when DNS settings are non-default.
- [x] #2 Proxy-based Xray test stages use the same DNS-aware probe options; default DNS still omits the DNS block.
- [x] #3 Tests verify generated probe JSON and native Xray compatibility where available.
- [x] #4 User-facing testing/configuration documentation explains that [dns] applies to managed runtimes and Xray probe tests.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Thread app DnsSettings into resolved Xray test generation options while preserving probe-only routing behavior. 2. Add real-delay/shared-probe generation tests, default omission coverage, and native Xray validation. 3. Update testing and configuration documentation. 4. Run focused tests and just fmt ci, then finalize this follow-up task.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Threaded DnsSettings into ResolvedTestSettings.gen_options, which is shared by real-delay, download, and upload Xray probes. ICMP/TCP remain direct checks and probe routing remains omitted. Updated testing, reference, architecture, and starter configuration docs. Validation: focused probe tests and native Xray validator passed; just fmt ci passed with 781 tests, clippy -D warnings, formatting, SQL/Markdown checks, and git diff --check.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Applied non-default [dns] settings to generated Xray probe configs used by real-delay, download, and upload testing, while preserving default omission and routing-free probes. Added wiring, generated JSON, default behavior, and native Xray validation coverage; updated user-facing docs. Verified with just fmt ci: 781 tests passed and clippy passed with -D warnings.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
