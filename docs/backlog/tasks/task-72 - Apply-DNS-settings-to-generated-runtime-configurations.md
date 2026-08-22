---
id: TASK-72
title: Apply DNS settings to generated runtime configurations
status: In Progress
assignee:
  - '@mhyrzt'
created_date: '2026-08-22 15:53'
updated_date: '2026-08-22 16:06'
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
- [ ] #1 DnsSettings map to generated Xray runtime configuration for managed sessions
- [ ] #2 sing-box path maps supported DNS options for both runtime usage and settings; anything unmapped is explicitly documented as out of scope
- [ ] #3 dns.* unavailable_reason gate removed from editor so TUI settings modal allows editing
- [ ] #4 Generation tests cover DNS config output and probe configs remain proxy-only unless routing requires DNS
- [ ] #5 User docs updated under docs/src for the dns section behavior
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add optional dns block to generated XrayConfig and thread DnsSettings through XrayGenOptions (runtime launch only, probes stay proxy-only) - verify: xray generation tests
2. Map app DnsSettings to Xray DnsObject in runtime_tuning.rs, skip emitting when at defaults - verify: mapping unit tests
3. Add dns object to SingboxConfig generation (servers, strategy mapping, disable_cache); document unmapped fields as Xray-only - verify: singbox generation tests
4. Wire dns into managed launch paths for both engines - verify: launch/runtime_service tests pass
5. Remove unavailable_reason gate from EditableSetting and TUI modal rendering - verify: editor/tui tests updated
6. Update docs/src/05-reference/config-file.md dns section with engine applicability - verify: md content review
<!-- SECTION:PLAN:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
