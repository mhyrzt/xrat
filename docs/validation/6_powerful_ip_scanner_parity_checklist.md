# Powerful IP Scanner Parity Checklist (xray-knife -> xrat)

This checklist maps gap area **#6 Powerful IP Scanner** from:

- `docs/validation/0_xray-knife_vs_xrat_gap_checklist.md`
- `../xray-knife/QA/6_powerful_ip_scanner.md`

---

## Scope and target behavior

Parity target for this area:

1. Cloudflare IP scanner command/service flow.
2. Candidate expansion + concurrent latency probing.
3. Optional top-N speedtest phase.
4. Resume semantics from DB/CSV and durable result persistence.
5. Optional proxy-config-assisted scan mode.

Out of scope for this checklist:

- runtime auto-rotation scheduler decisions (area #5),
- managed runtime supervisor daemon semantics (Phase 4.5).

---

## xray-knife reference map

Primary source files in `../xray-knife`:

- `cmd/cfscanner/cfscanner.go`
- `cmd/cfscanner/realityscanner.go`
- `pkg/scanner/scanner.go`
- `database/queries.go` (scanner upsert/load/resume paths)

Behavioral source narrative:

- `../xray-knife/QA/6_powerful_ip_scanner.md`

---

## Current state snapshot (xrat)

- Schema/repository groundwork exists:
  - `migrations/sqlite/0011_add_cf_scan_results.sql`
  - `migrations/postgres/0011_add_cf_scan_results.sql`
  - `src/db/repository/cf_scan_results.rs`
- Generic scan command exists but is not cfscanner parity:
  - `src/cli/scan.rs`
  - `src/app/commands/scan.rs`

Missing today:

- no cfscanner-specific command/service equivalent,
- no CIDR expansion + high-concurrency CF edge probe pipeline,
- no integrated resume strategy for scanner runs,
- no proxy-config-assisted scanner mode,
- no reality-specific scanner flow equivalent.

---

## Checklist

### `../xray-knife/QA/6_powerful_ip_scanner.md` alignment

- [ ] Add dedicated scanner command contract (subnets/input/output options).
- [ ] Implement candidate expansion from CIDRs/ranges/IP lists.
- [ ] Implement concurrent latency scan worker pool with bounded concurrency.
- [ ] Persist progressive/final scan results into `cf_scan_results`.
- [ ] Implement `--resume` behavior (DB-backed and/or CSV-backed skip set).
- [ ] Add optional speedtest stage for top candidates.
- [ ] Add sort policy parity (success -> latency -> speed tie-break).
- [ ] Add optional proxy-config-assisted scan mode (`--config` equivalent).
- [ ] Add reality-specific scanner flow if in-scope for xrat product target.
- [ ] Add export UX (CSV output contract and summaries).

Gap status summary:

- **PARTIAL**: persistence foundation exists.
- **MISSING**: scanner orchestration and CLI parity behavior.

---

## Suggested implementation order

1. [ ] Finalize scanner scope for xrat (latency-only vs latency+speedtest).
2. [ ] Build scanner service module and command UX (`scan cf` or dedicated cmd).
3. [ ] Wire DB persistence + resume behavior end-to-end.
4. [ ] Add optional proxy-assisted mode and reality extensions if approved.
5. [ ] Add performance/stability tests and benchmark-safe defaults.

---

## Exit criteria

- [ ] xrat can scan target IP ranges with concurrent latency probing.
- [ ] Scan results can be resumed and persisted reliably.
- [ ] Optional speedtest and proxy-assisted modes are implemented or explicitly
      documented non-goals.
- [ ] Scanner output and ranking semantics are documented and test-covered.

---

## Summary

- xray-knife area #6 is a full scanner subsystem.
- xrat currently has DB schema support for scanner results but lacks scanner
  runtime behavior.
- This area is best implemented after product-direction confirmation on scanner
  depth (especially speedtest/proxy-assisted/reality modes).
