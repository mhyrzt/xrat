# Storage + Persistence Parity Checklist (xray-knife -> xrat)

This checklist is detailed parity map for gap area **#2 Storage + Persistence**,
based on:

- `docs/validation/xray-knife_vs_xrat_gap_checklist.md`
- `../xray-knife/QA/2_storage_and_persistence.md`
- `../xray-knife/database/queries.go`
- `../xray-knife/database/migrations/0001_initial_schema.up.sql`
- `../xray-knife/cmd/root.go`
- `../xray-knife/cmd/subs/fetch.go`
- `../xray-knife/cmd/http/http.go`
- `../xray-knife/cmd/cfscanner/cfscanner.go`

---

## Scope and target behavior

Parity target for this phase:

1. Durable persistence for subscription ingest and fetched configs.
2. Durable persistence for connection test results with useful history
   semantics.
3. Schema + repository support for scanner-result persistence (if scanner in
   scope).
4. Clear DB path/bootstrap semantics at startup.

Out of scope for this phase:

- Parse/validation parity (covered by area #1).
- Proxy rotation/scanner runtime logic itself (covered by areas #5/#6).
- Runtime process behavior unrelated to persistence.

---

## Current state snapshot (xrat)

- DB backends:
  - SQLite + PostgreSQL supported (`src/db/connection.rs`, `src/db/schema.rs`,
    `src/app/config/database.rs`).
- Startup/path resolution:
  - DB path from `--database`, config (`[database.sqlite].path`), or app default
    (`src/cli/root.rs`, `src/app/path.rs`, `src/app/runtime.rs`).
- Subscription persistence:
  - Source rows inserted into `subscriptions`
    (`src/db/repository/subscriptions.rs`).
- Config persistence:
  - Batch import/upsert into `configs` by canonical `dedup_key`
    (`src/db/repository/configs.rs`,
    `migrations/*/0003_canonical_config_dedup_key.sql`).
- Test-result persistence:
  - Per-config rows stored in `connection_tests`
    (`src/db/repository/connection_tests.rs`).
- Test-run grouping persistence:
  - Run parent rows stored in `connection_test_runs` and linked from
    `connection_tests.run_id` (`migrations/*/0007_add_connection_test_runs.sql`,
    `src/db/repository/connection_tests.rs`).
- Runtime session persistence:
  - Session lifecycle stored in `runtime_sessions`
    (`src/db/repository/runtime_sessions.rs`).
- Missing compared to xray-knife storage features:
  - No `cf_scan_results` tables/repository.

---

## File-by-file delta checklist

## A) Database bootstrap + default path semantics

### `../xray-knife/cmd/root.go`, `../xray-knife/database/db.go`

- xray-knife initializes SQLite at startup with migration run and default path
  in user home.

### xrat parity files

- `src/cli/root.rs`
- `src/app/path.rs`
- `src/app/runtime.rs`
- `src/db/connection.rs`
- `src/db/schema.rs`

Checklist:

- [x] DB initializes on startup through runtime bootstrap and migration layer.
- [x] Default SQLite path resolution exists with CLI/config override precedence.
- [x] Migration execution is wired for both SQLite and PostgreSQL.
- [x] Document exact precedence table in user docs (`--database` vs config vs
      default path).

Gap notes:

- **PARTIAL** parity: behavior present, but path convention differs (xray-knife
  uses `~/.xray-knife/xray-knife.db`; xrat uses XRAT path defaults).

---

## B) Subscription source persistence

### `../xray-knife/database/queries.go`

- `AddSubscription`, `DeleteSubscription`, `UpdateSubscription`,
  `ListSubscriptions`, `GetSubscriptionByID`, `UpdateSubscriptionFetched`.

### xrat parity files

- `src/db/repository/subscriptions.rs`
- `src/db/repository/facade.rs`
- `src/app/commands/import.rs`
- `src/cli/list.rs`

Checklist:

- [x] Insert subscription/source record on import path.
- [x] List subscriptions with aggregate config counts.
- [x] Maintain source metadata (`source_kind`, `source_url`, optional name).
- [ ] Add explicit subscription CRUD parity (`add/rm/update` style command
      family).
- [ ] Add `last_fetched_at`-style tracking column and updates for fetch
      workflows.

Gap notes:

- **PARTIAL** parity: xrat stores source provenance but does not expose full
  standalone subscription-management command set like xray-knife `subs` group.

---

## C) Config ingest + upsert semantics

### `../xray-knife/database/queries.go`

- `UpsertSubscriptionConfigs` on `ON CONFLICT(config_link)`.

### xrat parity files

- `src/db/repository/configs.rs`
- `src/model/node_dedup_key.rs`
- `migrations/sqlite/0001_init.sql`
- `migrations/sqlite/0003_canonical_config_dedup_key.sql`

Checklist:

- [x] Batch config import/upsert exists.
- [x] DB-level uniqueness enforced.
- [x] Existing rows refreshed on conflict.
- [x] Dedup key semantics are deterministic and versioned.
- [x] Import returns summary (`imported_configs`, `total_configs`,
      `subscription_id`).
- [x] Decide if raw-link conflict key compatibility mode needed for cross-tool
      migration (`config_link`-style natural key).

Gap notes:

- **MATCHED (with stronger xrat design)**: same upsert class exists; xrat keying
  by canonical semantic hash is stricter than raw-link uniqueness.

---

## D) Connection test persistence model

### `../xray-knife/database/queries.go`

- `CreateHttpTestRun` + `InsertHttpTestResultsBatch` + history retrieval by
  latest run.

### xrat parity files

- `src/db/repository/connection_tests.rs`
- `src/db/model/connection_tests.rs`
- `migrations/sqlite/0001_init.sql`
- `migrations/sqlite/0002_add_connection_test_download_mbps.sql`

Checklist:

- [x] Persist per-config test outcomes.
- [x] Store key metrics/failure fields (`icmp`, `tcp`, `real_delay`,
      `download_mbps`, failure kind/reason).
- [x] Query test history by config ID and latest-by-config.
- [x] Add optional `test_runs` parent table for run-level grouping metadata.
- [x] Add run-level query UX equivalent to "latest run summary" semantics.
- [x] Add xray-knife-aligned HTTP fields (`ttfb`, `connect_ms`, `http_status`,
      `ip/location`) in same table.
- [x] Add optional upload-throughput persistence parity (`upload_mbps`) for
      direct schema-level alignment with xray-knife `http_test_results`.

Gap notes:

- **MATCHED** parity for run grouping: xrat now stores explicit run parent rows
  via `connection_test_runs` and links each `connection_tests` row by `run_id`.
- **MATCHED** schema metric parity: xrat now persists both `download_mbps` and
  `upload_mbps` in `connection_tests`.

---

## E) CF scanner persistence

### `../xray-knife/database/queries.go`

- `UpsertCfScanResultsBatch`, `GetCfScanResults`, `GetCfScanHistory`.

### xrat parity files

- N/A (no scanner persistence module yet)

Checklist:

- [x] Add `cf_scan_results` migration (SQLite + PostgreSQL).
- [x] Add repository with batch upsert by IP and history/recovery queries.
- [x] Define retention/index strategy for scanner rows.
- [ ] Integrate persistence hooks into future scanner command flow.

Gap notes:

- **PARTIAL** parity: scanner persistence schema + repository now present in
  xrat; command/runtime integration hooks remain pending.

---

## F) Schema comparison quick map

| Concern                   | xray-knife             | xrat                   | Status                        |
| ------------------------- | ---------------------- | ---------------------- | ----------------------------- |
| Subscription source table | `subscriptions`        | `subscriptions`        | **MATCHED**                   |
| Config table              | `subscription_configs` | `configs`              | **MATCHED (DIFFERENT MODEL)** |
| Config uniqueness key     | `config_link`          | `dedup_key`            | **DIFFERENT BY DESIGN**       |
| Test results table        | `http_test_results`    | `connection_tests`     | **MATCHED**                   |
| Test run grouping table   | `http_test_runs`       | `connection_test_runs` | **MATCHED**                   |
| Scanner results table     | `cf_scan_results`      | `cf_scan_results`      | **PARTIAL**                   |
| Runtime session table     | N/A                    | `runtime_sessions`     | **xrat extension**            |
| DB backend support        | SQLite only            | SQLite + PostgreSQL    | **xrat extension**            |

---

## Suggested implementation order (if strict parity desired)

1. [x] Add run-group persistence (`test_runs`) and tie `connection_tests` rows
       via FK.
2. [x] Add scanner persistence schema + repository API (`cf_scan_results`).
3. [x] Add subscription-management UX parity decisions
       (`subs add/rm/update/fetch` style vs keep current import-first UX).
4. [x] Add compatibility/import tooling for raw-link uniqueness migration if
       cross-tool DB migration needed.
5. [x] Update docs with DB path precedence and storage model differences.

---

## Follow-up (next session)

- [x] Expose persisted GeoIP fields in storage model: `endpoint_ip`,
      `endpoint_country`, `endpoint_asn`, `endpoint_location`.
- [x] Decide whether to add `upload_mbps` to `connection_tests` for strict
      xray-knife metric parity.
- [x] Move connection-testing UX parity items (geo filters, run-summary
      distribution, ping-loop behavior, status taxonomy) to area #3 checklist.
- [ ] Evaluate adding dedicated real-MMDB integration test for resolver priority
      (City -> Country -> ASN) to harden regression coverage.

---

## Exit criteria for "Area #2 complete"

- [x] xrat has explicit run-level test history grouping (or documented
      intentional non-goal).
- [x] scanner result persistence exists (or scanner out-of-scope decision
      documented).
- [x] subscription CRUD/fetch lifecycle parity decision documented.
- [x] HTTP metric parity decision documented (`upload_mbps`: add vs intentional
      non-goal).
- [x] storage model differences (`dedup_key` vs `config_link`) documented with
      migration guidance.
- [x] docs describe DB bootstrap/path precedence and backend differences
      clearly.

---

## Explicit parity decisions (May 9, 2026)

### DB path precedence (xrat)

Resolution order:

1. CLI `--database` value (highest priority).
2. Config file `[database.sqlite].path` (or PostgreSQL URL when backend is
   postgres).
3. XRAT default app path (lowest priority).

This differs from xray-knife fixed default (`~/.xray-knife/xray-knife.db`) and
is intentional for xrat multi-backend runtime.

### Subscription CRUD/fetch UX parity decision

- Decision: **intentional non-goal for now**.
- xrat keeps import-first UX (`import`, `list subscriptions/configs`) rather
  than adding xray-knife-style `subs add/rm/update/fetch` command family in this
  phase.
- Rationale: keep CLI surface small while scanner/runtime areas are still in
  active development.

### Config uniqueness compatibility decision (`dedup_key` vs `config_link`)

- Decision: **no default compatibility mode** in core storage path.
- xrat canonical `dedup_key` remains system-of-record uniqueness key.
- Migration guidance for cross-tool import:
  - re-parse legacy `config_link` rows into normalized node model,
  - recompute `dedup_key`,
  - import via normal upsert path,
  - keep raw link only as payload/trace data, not as primary uniqueness key.

### Scanner persistence retention/index strategy

- Table has `UNIQUE(ip)` upsert key with mutable freshness timestamp
  (`last_scanned_at`).
- Secondary indexes:
  - `last_scanned_at` for recency queries/resume windows,
  - `error` for healthy-vs-failed scan slicing,
  - `latency_ms` for best-candidate ranking.

## Parity verification pass (May 9, 2026)

Cross-check completed against:

- `../xray-knife/QA/2_storage_and_persistence.md`
- `../xray-knife/database/queries.go`
- `../xray-knife/database/migrations/0001_initial_schema.up.sql`
- `../xray-knife/cmd/root.go`
- `../xray-knife/cmd/subs/fetch.go`
- `../xray-knife/cmd/http/http.go`
- `../xray-knife/cmd/cfscanner/cfscanner.go`

Verified outcomes:

- DB bootstrap/default path behavior in xray-knife is home-dir fixed
  (`~/.xray-knife/xray-knife.db`); xrat remains override-first with XRAT
  defaults (**intentional divergence, doc gap remains**).
- Subscription flow parity still **partial**: xray-knife has explicit
  add/rm/update/fetch lifecycle + `last_fetched_at`; xrat currently import/list
  centric.
- HTTP history grouping now **matched**: xray-knife `http_test_runs` and xrat
  `connection_test_runs` both provide run parent rows.
- HTTP result metrics parity is **matched at persistence schema level**:
  xray-knife and xrat both persist `download_mbps` and `upload_mbps`.
- Scanner persistence parity remains **missing**: xray-knife has
  `cf_scan_results` upsert/history path; xrat now has equivalent table/repo, but
  scanner command integration is still pending.

---

## Summary

- xrat already has strong storage fundamentals: migrations, durable import
  upsert, and structured per-config test persistence.
- Main parity gap versus xray-knife storage is scanner command-flow integration
  with persisted `cf_scan_results`.
- xrat intentionally diverges in key areas (`dedup_key`, runtime session table,
  PostgreSQL support) and these are net strengths if documented clearly.
