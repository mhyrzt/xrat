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
- Basic scanner command/service exists (partial parity):
  - `src/cli/scan.rs`
  - `src/app/commands/scan.rs`
  - supports `--ips`, `--file`, `--port`, `--timeout`, `--history`
  - performs TCP latency probes and persists results to `cf_scan_results`
  - supports scanner history queries from durable storage

Missing today:

- no cfscanner-specific command/service equivalent,
- no CIDR expansion + high-concurrency CF edge probe pipeline,
- no integrated resume strategy for scanner runs,
- no proxy-config-assisted scanner mode,
- no reality-specific scanner flow equivalent.

---

## Checklist

### `../xray-knife/QA/6_powerful_ip_scanner.md` alignment

- [x] Add dedicated scanner command contract (subnets/input/output options).
- [ ] Implement candidate expansion from CIDRs/ranges/IP lists.
- [ ] Implement concurrent latency scan worker pool with bounded concurrency.
- [x] Persist progressive/final scan results into `cf_scan_results`.
- [ ] Implement `--resume` behavior (DB-backed and/or CSV-backed skip set).
- [ ] Add optional speedtest stage for top candidates.
- [ ] Add sort policy parity (success -> latency -> speed tie-break).
- [ ] Add optional proxy-config-assisted scan mode (`--config` equivalent).
- [ ] Add reality-specific scanner flow if in-scope for xrat product target.
- [ ] Add export UX (CSV output contract and summaries).

Gap status summary:

- **PARTIAL**: CLI scanner + persistence path implemented.
- **MISSING**: cfscanner-grade candidate generation, concurrency, resume,
  speedtest, and proxy-assisted/reality extensions.

---

## Suggested implementation order

1. [x] Finalize scanner scope for xrat (latency-only vs latency+speedtest).
       **Decision (2026-05-28):** Current scope is latency-only TCP probing with
       persistence. CIDR expansion, bounded concurrency, resume, speedtest, and
       proxy-assisted modes remain backlog items pending product decision.
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
- [x] Basic scanner command can probe explicit IP candidates and persist/query
      results.
- [x] Scanner IP input parsing and dedup have test coverage.

---

## Scope Decision (2026-05-28)

Current scanner scope is **latency-only TCP probing**:

- `xrat scan --ips <ip1>,<ip2>` or `--file <path>` for input
- Sequential TCP connect probes with configurable timeout
- IP dedup via `BTreeSet` before probing
- Results persisted to `cf_scan_results` with `UNIQUE(ip)` upsert
- `xrat scan --history <n>` for querying persisted results

**Deferred (pending product decision):**

- CIDR/range expansion (no `cidr`/`IpNet` dependency)
- Bounded concurrent worker pool (currently sequential)
- Resume semantics (every invocation scans all provided IPs)
- Speedtest phase (`download_mbps`/`upload_mbps` schema fields exist but are
  always `None` from scanner)
- Proxy-config-assisted scan
- Reality-specific scanner flow
- CSV/export UX and ranking summaries

---

## Summary

- xray-knife area #6 is a full scanner subsystem.
- xrat now has a basic scanner runtime (`xrat scan`) and durable persistence,
  but does not yet match cfscanner depth.
- Current scope is latency-only; advanced features remain backlog pending
  product-direction confirmation.
