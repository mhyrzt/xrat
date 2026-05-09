# Connection Testing + Ping Parity Checklist (xray-knife -> xrat)

This checklist maps gap area **#3 Connection Testing + Ping** using:

- `docs/validation/0_xray-knife_vs_xrat_gap_checklist.md`
- `../xray-knife/QA/3_connection_testing_and_ping.md`
- `../xray-knife/cmd/http/http.go`
- `../xray-knife/pkg/http/examiner.go`
- `../xray-knife/cmd/net/icmp.go`
- `../xray-knife/cmd/net/tcp.go`
- `src/cli/test.rs`
- `src/app/commands/test.rs`
- `src/tester/icmp.rs`
- `src/tester/tcp.rs`
- `src/tester/real_delay.rs`
- `src/tester/download.rs`

---

## Scope and target behavior

Parity target for this phase:

1. Batch/single config test parity with persistence.
2. Useful metric parity for HTTP-like probe fields.
3. Ping-loop parity decision (`--ping` continuous loop).
4. Comparable status and reporting semantics.

Out of scope for this phase:

- Auto-rotation scheduler behavior (area #5).
- Scanner command/runtime logic (area #6).

---

## Current state snapshot (xrat)

- Test pipeline:
  - staged ICMP -> TCP gate -> real-delay -> download
    (`src/app/commands/test.rs`).
- Primitive checks:
  - ICMP and TCP probes implemented (`src/tester/icmp.rs`, `src/tester/tcp.rs`).
- Batch/single test UX:
  - `xrat test [id]` with bulk filters/format/sort options
    (`src/cli/test.rs`).
- Persistence:
  - test runs + per-result rows in DB
    (`connection_test_runs`, `connection_tests`).
- Persisted metrics:
  - `icmp/tcp/real_delay`, `download_mbps`, `upload_mbps`,
    `ttfb_ms`, `connect_ms`, `http_status`, endpoint IP/location/country/ASN.
- Latest summary UX:
  - `--latest-run-summary` with optional `--country` / `--asn` filters and
    country/ASN distribution output.
- Main missing piece:
  - no continuous ping mode (`test --ping`) equivalent to xray-knife loop.

---

## File-by-file delta checklist

## A) Core config examination flow

### `../xray-knife/pkg/http/examiner.go`

- HTTP-centric examine path with delay, connect, ttfb, status, speed, IP info.

### xrat parity files

- `src/app/commands/test.rs`
- `src/tester/real_delay.rs`
- `src/tester/download.rs`

Checklist:

- [x] Single config examination path exists.
- [x] Batch examination path exists with configurable concurrency.
- [x] Persist and print delay/speed/failure outcomes.
- [x] Persist HTTP-aligned fields (`ttfb_ms`, `connect_ms`, `http_status`).
- [x] Persist endpoint metadata fields (`endpoint_ip`, `endpoint_location`,
      `endpoint_country`, `endpoint_asn`).
- [ ] Add true upload-speed measurement path (currently persistence field exists,
      command flow stores `upload_mbps = None`).

Gap notes:

- **PARTIAL** parity: field-level model mostly aligned; upload measurement and
  some xray-knife examiner semantics still differ.

---

## B) CLI test command surface

### `../xray-knife/cmd/http/http.go`

- batch test flags, DB source mode, outputs, optional ping-loop mode.

### xrat parity files

- `src/cli/test.rs`
- `src/app/commands/test.rs`

Checklist:

- [x] Bulk filter flags (`--enabled-only`, `--subscription`, etc.).
- [x] Output modes (`tsv`, `csv`, `json`) and sorting controls.
- [x] Run-level summary command path (`--latest-run-summary`).
- [x] Geo filters for latest run summary (`--country`, `--asn`).
- [ ] Add optional continuous ping loop (`test --ping`) + Ctrl+C summary stats.

Gap notes:

- **PARTIAL** parity: main missing behavioral gap is ping-loop command mode.

---

## C) Status taxonomy + reporting semantics

### `../xray-knife/pkg/http/examiner.go`

- richer result labels (`passed`, `semi-passed`, `timeout`, `broken`, etc.).

### xrat parity files

- `src/app/commands/test.rs`

Checklist:

- [x] Stable status model exists (`ok`, `failed`, `skipped`).
- [x] Failure kind/reason classification persisted.
- [ ] Decide whether to add compatibility mapping table for
      xray-knife-style statuses (if cross-tool report parity needed).

Gap notes:

- **PARTIAL** parity: xrat status model simpler by design.

---

## D) Net probe primitive parity

### `../xray-knife/cmd/net/icmp.go`, `../xray-knife/cmd/net/tcp.go`

### xrat parity files

- `src/tester/icmp.rs`
- `src/tester/tcp.rs`

Checklist:

- [x] ICMP primitive probe available.
- [x] TCP primitive probe available.
- [x] Integrated into bulk/single staged test flow.

Gap notes:

- **MATCHED** parity for primitive checks.

---

## Suggested implementation order (remaining)

1. [ ] Add `test --ping` continuous loop mode.
2. [ ] Decide/upload real measurement path for `upload_mbps`.
3. [ ] Add optional status vocabulary compatibility mapping for cross-tool
       exported reports.

---

## Exit criteria for "Area #3 complete"

- [ ] Ping loop parity decision implemented or explicitly documented non-goal.
- [ ] Upload metric parity decision implemented or explicitly documented
      non-goal.
- [ ] Status taxonomy compatibility decision documented.
- [x] Core test persistence model aligned for major HTTP + geo fields.

---

## Summary

- xrat already matches most connection-testing storage/reporting needs.
- Major remaining behavior gap is continuous ping mode.
- Status vocabulary remains intentionally simplified unless cross-tool
  comparability becomes hard requirement.
