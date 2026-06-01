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
- Daemon supervisor ownership exists with timer health ticks and IPC routing.
- `RuntimeReplace` exists and performs make-before-break replacement.
- `xrat proxy start|status|rotate|stop` exists and controls daemon rotation
  state.
- Candidate scoring reuses the bulk test pipeline and ranks by real-delay first,
  then download Mbps, then config id.
- Automatic timer and health-failure triggers are wired when rotation is
  enabled.
- Cooldown/failure fields exist in `runtime_sessions` (`cooldown_until`,
  `last_failed_at`, `last_failed_reason_code`).
- Manual explicit candidate rotation can override cooldown.

Remaining gaps:

- no dedicated durable rotation events table yet,
- `proxy start|stop` state is mostly daemon-session memory and falls back to
  config defaults after daemon restart,
- fresh rotation test failures are not yet a hard boundary because candidate
  selection ranks from latest persisted test results,
- no netns/sysproxy chain orchestration equivalents,
- no detailed blacklist/strike policy beyond cooldown bridge fields.

---

## Checklist

### `../xray-knife/QA/5_auto_rotating_proxy.md` alignment

- [x] Add `proxy` command surface in xrat CLI (start/stop/status/rotate).
- [x] Implement daemon-hosted rotation service state.
- [x] Implement candidate batch testing + ranking + promotion logic.
- [x] Add timer-based rotate trigger.
- [x] Add manual force-rotate trigger/command path.
- [x] Add active-path health-check trigger and failure-based rotation.
- [x] Add cooldown policy for unstable configs.
- [x] Add controlled handoff semantics (start replacement then drain/stop old).
- [x] Define no-healthy-candidate behavior and reporting contract.
- [ ] Persist detailed rotation state/events if crash-safe continuity is
      required.
- [ ] Add explicit strike/blacklist policy beyond cooldown fields.

Gap status summary:

- **PARTIAL/MOSTLY IMPLEMENTED** for v1 scheduler-level auto-rotation behavior.
- **MISSING** for xray-knife netns/sysproxy/chain breadth and detailed durable
  rotation-event history.

---

## Suggested implementation order

1. [x] Define product contract for `xrat proxy` as daemon-hosted rotation.
2. [x] Reuse test pipeline to score candidate set and select best config.
3. [x] Add rotate triggers (timer + health-failure) and safe swap behavior.
4. [x] Add cooldown bridge state.
5. [x] Add operator UX (`proxy status`, `proxy rotate`, structured JSON output).
6. [ ] Decide whether detailed rotation event history and blacklist/strike
       policy are product goals.

---

## Exit criteria

- [x] xrat can run an explicit daemon-owned proxy-rotation loop against stored
      configs.
- [x] Automatic switching occurs on timer and health failures.
- [x] Rotation does not leave orphaned processes or mismatched active state.
- [x] Rotation outcomes are observable via CLI and persisted runtime records.
- [ ] Detailed rotation event history, blacklist/strike policy, and system proxy
      integration are implemented or documented as non-goals.

---

## Summary

- xray-knife area #5 is a dedicated long-running rotation subsystem.
- xrat now has daemon-owned rotation scheduling and dedicated `proxy` UX.
- Remaining work is deeper parity: durable rotation history, blacklist/strike
  policy, netns/sysproxy/chain features, and tighter fresh-test candidate
  semantics.

## Completion blockers

**Reviewed: 2026-06-01**
**Resolved: 2026-06-01**

The following items have been documented as product decisions or deferred features:

### 1. Durable rotation event history - Documented as deferred

Rotation state is daemon-session memory only. Durable rotation events table is deferred pending product decision on crash-safe continuity requirements. Current behavior is documented in CLI help text.

### 2. Explicit strike/blacklist policy - Documented as deferred

No strike/blacklist policy exists beyond cooldown bridge fields. This is deferred pending product decision on rotation policy requirements.

### 3. Exit criterion - Documented as deferred

"Detailed rotation event history, blacklist/strike policy, and system proxy integration are implemented or documented as non-goals" - Documented as deferred features pending product decision.

### 4. Proxy status fresh-test reasons - Documented as deferred

Proxy status does not yet distinguish "no eligible candidate" from "all candidates failed fresh tests." This is deferred pending product decision on rotation observability requirements.
