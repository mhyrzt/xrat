# Auto-Rotating Proxy Parity Checklist (xray-knife -> xrat)

This checklist maps gap area **#5 Auto-Rotating Proxy** from:

- `docs/validation/0_xray-knife_vs_xrat_gap_checklist.md`
- `../xray-knife/QA/5_auto_rotating_proxy.md`

---

## Scope and target behavior

Parity target for this area:

1. Long-running rotation service that can keep one active proxy and switch
   candidates automatically.
2. Candidate selection pipeline (batch test, rank, promote best candidate).
3. Rotation triggers (time-based, manual force rotate, health-failure based).
4. Stability controls (blacklist, cooldown, drain/swap semantics, no-healthy
   fallback handling).

Out of scope for this checklist:

- full scanner subsystem parity (covered by area #6),
- broad multi-engine runtime parity decisions (covered by area #4),
- system-level daemon packaging (`systemd`, container manifests).

---

## xray-knife reference map

Primary source files in `../xray-knife`:

- `pkg/proxy/service.go`
- `cmd/proxy/proxy.go`
- `pkg/proxy/netns/*`
- `pkg/proxy/sysproxy/*`
- `pkg/proxy/chain.go`

Behavioral source narrative:

- `../xray-knife/QA/5_auto_rotating_proxy.md`

---

## Current state snapshot (xrat)

- Runtime control exists for one managed session:
  - `xrat connect <id>` / `disconnect` / `status`
  - `src/app/runtime_service.rs`
- Session replacement policy exists:
  - `[runtime].replace_active_session`
- Command-driven stale-state reconciliation exists:
  - stale PID/session handling in `src/app/runtime_service.rs`

Missing today:

- no `proxy` command with long-running rotation loop,
- no scheduler that continuously re-tests candidates and rotates active proxy,
- no blacklist/cooldown/drain rotation state model,
- no netns/sysproxy chain orchestration equivalents.

---

## Checklist

### `../xray-knife/QA/5_auto_rotating_proxy.md` alignment

- [ ] Add `proxy` command surface in xrat CLI (start/stop/status/rotate).
- [ ] Implement long-running rotation service (`runRotationMode` equivalent).
- [ ] Implement candidate batch testing + ranking + promotion logic.
- [ ] Add timer-based rotate trigger (`--rotate` style semantics).
- [ ] Add manual force-rotate trigger/command path.
- [ ] Add active-path health-check trigger and failure-based rotation.
- [ ] Add strike/blacklist/cooldown policy for unstable configs.
- [ ] Add controlled handoff semantics (start replacement then drain/stop old).
- [ ] Define no-healthy-candidate behavior and reporting contract.
- [ ] Persist rotation state/events if feature needs crash-safe continuity.

Gap status summary:

- **MISSING** for scheduler-level auto-rotation behavior.
- **PARTIAL FOUNDATION** from Phase 4 runtime lifecycle building blocks.

---

## Suggested implementation order

1. [ ] Define product contract for `xrat proxy` (foreground loop first).
2. [ ] Reuse test pipeline to score candidate set and select best config.
3. [ ] Add rotate triggers (timer + health-failure) and safe swap behavior.
4. [ ] Add blacklist/cooldown state in memory, then persist if needed.
5. [ ] Add operator UX (`proxy status`, `proxy rotate`, structured JSON output).

---

## Exit criteria

- [ ] xrat can run an explicit proxy-rotation loop against stored configs.
- [ ] Automatic switching occurs on timer and health failures.
- [ ] Rotation does not leave orphaned processes or mismatched active state.
- [ ] Rotation outcomes are observable via CLI and persisted runtime records.

---

## Summary

- xray-knife area #5 is a dedicated long-running rotation subsystem.
- xrat currently has managed single-session runtime control, but no scheduler.
- Phase 4.5 decisions (supervisor model) should be finalized before deep
  rotation implementation so process ownership remains consistent.
