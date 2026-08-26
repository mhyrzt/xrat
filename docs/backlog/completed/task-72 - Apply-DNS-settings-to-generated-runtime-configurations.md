---
id: TASK-72
title: Apply DNS settings to generated runtime configurations
status: Done
assignee:
  - '@mhyrzt'
created_date: '2026-08-22 15:53'
updated_date: '2026-08-22 20:09'
labels: []
dependencies: []
ordinal: 33000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
DNS settings ([dns] section, src/app/config/dns.rs) parse and persist in config.toml but are never consumed by the Xray/sing-box runtime config generators. Because of this EditableSetting::unavailable_reason (src/app/config/editor.rs:148) disables all dns.* fields in the TUI settings modal with reason 'DNS settings are not yet applied to generated runtime configurations'. Wire DnsSettings into runtime generation for BOTH engines (Xray and sing-box) so DNS is actually used at runtime, following the same pattern used for mux (task-13) and fragmentation (task-12), then remove the unavailable_reason gate so DNS becomes editable in the TUI settings modal.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 DnsSettings map to generated Xray runtime configuration for managed sessions
- [x] #2 sing-box path maps supported DNS options for both runtime usage and settings; anything unmapped is explicitly documented as out of scope
- [x] #3 dns.* unavailable_reason gate removed from editor so TUI settings modal allows editing
- [x] #4 Generation tests cover DNS config output and probe configs remain proxy-only unless routing requires DNS
- [x] #5 User docs updated under docs/src for the dns section behavior
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Implement DNS generation for managed runtimes without coupling engine output types to app config.

1. Add optional DNS output types/fields to generated Xray and sing-box configs. Thread Xray DNS through managed launch only; keep Xray probe options and sing-box probe generation DNS-free. Xray output uses documented servers, hosts, queryStrategy, useSystemHosts, disableCache, disableFallback, and enableParallelQuery fields, omitting only the whole block for application defaults.
2. Map sing-box through modern typed DNS servers. Support UseIPv4/UseIPv6, documented UDP/TCP/TLS/QUIC/HTTPS/local server forms, hostname domain_resolver fields, exact/full host keys, system-host path behavior, explicit final, and disable_cache. Reject UseIP/UseSystem when emitting non-default sing-box DNS, non-exact host prefixes, malformed/legacy-only server forms, and all unmapped fallback/parallel semantics before process launch.
3. Remove the dns.* TUI unavailable gate and update settings tests/help. Update configuration, TUI, template, and config-generation documentation with engine-specific support and limitations.
4. Add mapping, generated JSON, probe isolation, launch error-ordering, TUI, and native-validator coverage. Run focused tests, native xray/sing-box checks where available, and just fmt ci.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented managed-runtime DNS generation: Xray emits the documented DNS object; sing-box emits typed DNS servers, exact hosts, system-host handling, explicit final, and disable_cache. Unsupported sing-box strategy, host syntax, server forms, fallback, and parallel-query settings fail before process launch. Probe configs remain DNS-free. Added native xray/sing-box validator coverage where binaries are installed. Validation: just fmt ci passed; 780 tests passed; clippy -D warnings passed; git diff --check passed.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-08-22 19:37
---
Implementation started from the approved plan; preserving probe-only behavior and validating generated JSON against native cores.
---
<!-- COMMENTS:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Applied [dns] settings to managed Xray/V2Ray and sing-box runtime JSON, removed the TUI unavailable gate, documented engine-specific support and strict limitations, and added generation, launch, probe-isolation, UI, and native-validator tests. Verified with just fmt ci: 780 tests passed, clippy passed with -D warnings, and native Xray/sing-box config validation passed.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
