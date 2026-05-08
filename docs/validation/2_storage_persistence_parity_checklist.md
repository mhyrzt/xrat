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
- Runtime session persistence:
  - Session lifecycle stored in `runtime_sessions`
    (`src/db/repository/runtime_sessions.rs`).
- Missing compared to xray-knife storage features:
  - No `cf_scan_results` tables/repository.
  - No explicit `http_test_runs` parent table for grouping test runs.

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
- [ ] Document exact precedence table in user docs (`--database` vs config vs
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
- [ ] Decide if raw-link conflict key compatibility mode needed for cross-tool
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
- [x] Add xray-knife-aligned HTTP fields (`ttfb`, `connect_ms`,
      `http_status`, `ip/location`) in same table.

Gap notes:

- **PARTIAL** parity: xrat stores robust per-config records but does not group
  rows under explicit run IDs.

---

## E) CF scanner persistence

### `../xray-knife/database/queries.go`

- `UpsertCfScanResultsBatch`, `GetCfScanResults`, `GetCfScanHistory`.

### xrat parity files

- N/A (no scanner persistence module yet)

Checklist:

- [ ] Add `cf_scan_results` migration (SQLite + PostgreSQL).
- [ ] Add repository with batch upsert by IP and history/recovery queries.
- [ ] Define retention/index strategy for scanner rows.
- [ ] Integrate persistence hooks into future scanner command flow.

Gap notes:

- **MISSING** parity: scanner persistence layer not present in xrat.

---

## F) Schema comparison quick map

| Concern                   | xray-knife             | xrat                | Status                        |
| ------------------------- | ---------------------- | ------------------- | ----------------------------- |
| Subscription source table | `subscriptions`        | `subscriptions`     | **MATCHED**                   |
| Config table              | `subscription_configs` | `configs`           | **MATCHED (DIFFERENT MODEL)** |
| Config uniqueness key     | `config_link`          | `dedup_key`         | **DIFFERENT BY DESIGN**       |
| Test results table        | `http_test_results`    | `connection_tests`  | **PARTIAL**                   |
| Test run grouping table   | `http_test_runs`       | N/A                 | **MISSING**                   |
| Scanner results table     | `cf_scan_results`      | N/A                 | **MISSING**                   |
| Runtime session table     | N/A                    | `runtime_sessions`  | **xrat extension**            |
| DB backend support        | SQLite only            | SQLite + PostgreSQL | **xrat extension**            |

---

## Suggested implementation order (if strict parity desired)

1. [x] Add run-group persistence (`test_runs`) and tie `connection_tests` rows
       via FK.
2. [ ] Add scanner persistence schema + repository API (`cf_scan_results`).
3. [ ] Add subscription-management UX parity decisions
       (`subs add/rm/update/fetch` style vs keep current import-first UX).
4. [ ] Add compatibility/import tooling for raw-link uniqueness migration if
       cross-tool DB migration needed.
5. [ ] Update docs with DB path precedence and storage model differences.

---

## Follow-up (next session)

- [ ] Expose persisted GeoIP fields in CLI test/list UX:
      `endpoint_ip`, `endpoint_country`, `endpoint_asn`,
      `endpoint_location`.
- [ ] Add query/filter support for persisted test geography metadata
      (e.g. country/ASN filters).
- [ ] Add run-summary UX enrichment that includes country/ASN distribution for
      latest run.
- [ ] Evaluate adding dedicated real-MMDB integration test for resolver
      priority (City -> Country -> ASN) to harden regression coverage.

---

## Exit criteria for "Area #2 complete"

- [ ] xrat has explicit run-level test history grouping (or documented
      intentional non-goal).
- [ ] scanner result persistence exists (or scanner out-of-scope decision
      documented).
- [ ] subscription CRUD/fetch lifecycle parity decision documented.
- [ ] storage model differences (`dedup_key` vs `config_link`) documented with
      migration guidance.
- [ ] docs describe DB bootstrap/path precedence and backend differences
      clearly.

---

## Summary

- xrat already has strong storage fundamentals: migrations, durable import
  upsert, and structured per-config test persistence.
- Main parity gaps versus xray-knife storage are run-group test history and CF
  scanner result persistence.
- xrat intentionally diverges in key areas (`dedup_key`, runtime session table,
  PostgreSQL support) and these are net strengths if documented clearly.
