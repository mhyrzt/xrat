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
- `src/tester/upload.rs`

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
  - staged ICMP -> TCP gate -> real-delay -> download -> optional upload
    (`src/app/commands/test.rs`).
- Primitive checks:
  - ICMP and TCP probes implemented (`src/tester/icmp.rs`, `src/tester/tcp.rs`).
- Batch/single test UX:
  - `xrat test [id]` with bulk filters/format/sort options (`src/cli/test.rs`).
- Persistence:
  - test runs + per-result rows in DB (`connection_test_runs`,
    `connection_tests`).
- Persisted metrics:
  - `icmp/tcp/real_delay`, `download_mbps`, `upload_mbps`, `ttfb_ms`,
    `connect_ms`, `http_status`, endpoint IP/location/country/ASN.
- Latest summary UX:
  - `--latest-run-summary` with optional `--country` / `--asn` filters and
    country/ASN distribution output.
- Main behavioral gap from earlier pass:
  - continuous ping mode (`test --ping`) was missing; now implemented with
    Ctrl+C loop summary and persisted `ping_loop` run grouping.
  - upload speed path was previously deferred; now implemented via
    `--upload-url` (persisted to `upload_mbps`).

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
- [x] Add true upload-speed measurement path (enabled by `--upload-url`,
      persisted into `connection_tests.upload_mbps`).

Gap notes:

- **MATCHED (core metrics)** parity: delay/download/upload + HTTP/geo fields are
  persisted; some examiner implementation details still differ.

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
- [x] Add optional continuous ping loop (`test --ping`) + Ctrl+C summary stats.

Gap notes:

- **MATCHED** parity for ping-loop behavior (`--ping` + Ctrl+C summary).

---

## C) Status taxonomy + reporting semantics

### `../xray-knife/pkg/http/examiner.go`

- richer result labels (`passed`, `semi-passed`, `timeout`, `broken`, etc.).

### xrat parity files

- `src/app/commands/test.rs`

Checklist:

- [x] Stable status model exists (`ok`, `failed`, `skipped`).
- [x] Failure kind/reason classification persisted.
- [x] Decide whether to add compatibility mapping table for xray-knife-style
      statuses (if cross-tool report parity needed).

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

1. [x] Add `test --ping` continuous loop mode.
2. [x] Implement upload measurement path for `upload_mbps` (`--upload-url` +
       persistence).
3. [x] Add optional status vocabulary compatibility mapping for cross-tool
       exported reports (decision: keep simplified xrat status model for now).

---

## Exit criteria for "Area #3 complete"

- [x] Ping loop parity implemented (`test --ping`).
- [x] Upload metric parity implemented (`upload_mbps` measured + persisted).
- [x] Status taxonomy compatibility decision documented.
- [x] Core test persistence model aligned for major HTTP + geo fields.

---

## Explicit parity decisions (May 9, 2026)

- Upload metric parity:
  - Decision updated: **implemented** in current test flow.
  - Runtime measurement is opt-in via `--upload-url`; field remains nullable
    when upload stage is not enabled.

---

## Summary

- xrat already matches most connection-testing storage/reporting needs.
- Ping-loop parity now implemented via `xrat test <id> --ping`.
- Upload metric path now implemented via
  `xrat test <id> --upload-url <http-endpoint>`.
- Status vocabulary remains intentionally simplified unless cross-tool
  comparability becomes hard requirement.
