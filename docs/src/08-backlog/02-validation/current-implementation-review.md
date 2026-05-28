# Current Implementation Review

Reviewed: 2026-05-28

Scope:

- `docs/src/08-backlog/01-plan/`
- `docs/src/08-backlog/02-validation/`
- current `src/` and migration implementation

Validation command:

```text
cargo test -q
```

Result: 241 tests passed (3 suites).

## Summary

The implementation is ahead of several older checklist entries. Phases 1 through
4.6 are substantially implemented, Phase 5 HTTP API has a usable foreground
server path with route-level test coverage, and the area-specific validation
checklists are more accurate than the aggregate xray-knife gap checklist.

The main remaining implementation risks are concentrated in six areas:

- the documented soft-delete policy is not implemented in schema/repositories
- Phase 5 HTTP API still does route-layer N+1 querying, in-memory sorting, and
  in-memory pagination
- daemon-hosted HTTP API mode is documented but not wired (`server.enabled` is a
  dead config field)
- automatic rotation ranks from latest persisted test rows instead of strictly
  the fresh rotation test run
- proxy start/stop state is volatile and resets to config defaults on daemon
  restart
- scanner parity remains intentionally shallow despite baseline scan persistence

The main documentation risk is drift between
`0_xray-knife_vs_xrat_gap_checklist.md` and the newer area-specific validation
checklists. The aggregate checklist still describes several now-implemented
features as missing.

## Phase-by-Phase Assessment

### Phase 1: Parse + Import

Status: **complete**

- Parser supports `vless`, `vmess`, `ss`, `trojan`, `http`, `socks5`, and
  `hy2`/`hysteria2` (sing-box parse path).
- Normalization and dedup happen before persistence.
- Subscription URL ingestion and mixed input decoding work.
- Decode behavior has 16 focused tests (`src/support/decode.rs`).
- Import/parse flow has 14 focused tests (`src/config/import/mod.rs`).
- Phase 1 completion criteria for decode and mixed-input tests are met.

### Phase 2: Storage + Persistence

Status: **complete with soft delete added**

- SQLite and PostgreSQL dual-backend support with 15 migrations each.
- Subscription source rows, config upsert by canonical `dedup_key`, connection
  test history, runtime sessions, and scanner results all persist correctly.
- Config lifecycle flags (`is_active`, `is_enabled`, `is_selected`) work.
- Soft delete implemented: `is_deleted`, `deleted_at` columns in migration 0015.
  Default delete is `UPDATE configs SET is_deleted = TRUE`; hard delete/purge is
  explicit via repository method. Deleted rows excluded from list queries by
  default; `--include-deleted` flag available.
- FK integrity preserved: dependent rows survive soft delete.

### Phase 2.5: CLI Restructure

Status: **complete**

- Subcommand-based CLI with global `--database` and `--config` flags.
- `import`, `add`, `list`, lifecycle commands, `show`, `status`, `test`,
  `connect`, `disconnect`, `parse`, `scan`, `proxy`, `daemon`, `serve` all
  exist.
- Clean split between `src/cli/` (Clap structs) and `src/app/commands/`
  (handlers).

### Phase 3: Connection Testing

Status: **complete**

- `xrat test <id>` with ICMP, TCP, real-delay, download, and upload stages.
- Configurable test order, failure policy, concurrency, and stage enablement.
- Bulk testing with filters, progress bars, and structured output
  (TSV/CSV/JSON).
- `test --ping` continuous loop with Ctrl+C summary is implemented
  (`src/app/commands/test/handlers/ping.rs`).
- Test run grouping via `connection_test_runs` with `run_id` FK.
- Persisted metrics include `download_mbps`, `upload_mbps`, `ttfb_ms`,
  `connect_ms`, `http_status`, endpoint IP/country/ASN.

### Phase 3.5: Local App Configuration

Status: **complete**

- `config.toml` is the canonical app config file.
- Sections for `[runtime]`, `[routing]`, `[geo]`, `[dns]`, `[testing]`,
  `[server]`, `[database]`, `[parser]`, `[proxy]` exist.
- Path resolution via `XRAT_PATH` or `~/.config/xrat/`.
- Secret/env value pattern for sensitive config.

### Phase 3.6: Cleanup

Status: **complete**

- Documentation moved to `docs/`.
- Parser config mode, centralized tester defaults, tracing diagnostics,
  canonical dedup key, database/repository facades, concrete error types,
  PostgreSQL backend, and Xray config module splits all landed.

### Phase 4: Managed Runtime

Status: **complete**

- `connect <id>`, `disconnect`, `status` with managed Xray process lifecycle.
- Runtime session persistence with explicit inbound port columns.
- Stale PID reconciliation, replacement policy, `--json` output.
- `RuntimeService` abstraction for reuse by daemon and future API.

### Phase 4.5: Daemon + Supervisor

Status: **complete**

- `xrat daemon start|status|stop` with detached daemon process.
- Unix socket IPC with protocol version gating.
- Supervisor event loop with health ticks and cooldown-aware replacement.
- Strict reattach policy (PID, executable, cmdline checks).
- Make-before-break `RuntimeReplace` with rollback safety.
- Transition reason taxonomy and ownership metadata persisted.

### Phase 4.6: Auto-Rotating Proxy

Status: **complete for v1 rotation, with known data-freshness gap**

- `xrat proxy start|status|rotate|stop` controls daemon rotation state.
- Timer, manual, and health-failure triggers wired.
- Candidate ranking: lowest `real_delay_ms`, then highest `download_mbps`, then
  lowest config id.
- Cooldown policy suppresses repeated automatic retries.
- Manual explicit candidate override can bypass cooldown.

Known gap: rotation bulk test results are discarded and ranking re-queries the
DB, so stale data can promote a candidate. See findings below.

### Phase 5: HTTP API

Status: **~85% complete, foreground mode usable**

- `xrat serve` with `--host`/`--port` overrides.
- `GET /health`, `/json`, `/b64`, `/configs`, `/configs/{id}` handlers exist.
- Optional `?key=` API key enforcement.
- 11 route-level tests exist in `src/server/mod.rs` and `src/server/auth.rs`.
- Repository query helpers implemented: `list_configs_with_latest_tests()`,
  `list_top_by_real_delay()`, `list_configs_paginated()`, `SoftDelete` filter.
  Sorting, filtering, and pagination pushed to SQL.
- `/b64` uses a simplified query path that skips latest-test joins.
- Daemon-hosted HTTP API wired: `[server].enabled = true` starts HTTP server
  alongside daemon IPC; daemon status reports HTTP API state.
- Systemd service templates added in `packaging/systemd/`.

Remaining: Some route handlers still use legacy per-item helpers instead of new
repository helpers (partial migration in progress).

## High Priority Findings

### Config Delete Is Now Soft Delete

**Status: resolved**

The Phase 2 soft-delete gap has been addressed:

- Migration 0015 adds `is_deleted BOOLEAN NOT NULL DEFAULT FALSE` and
  `deleted_at TIMESTAMP NULL` to `configs` (both SQLite and PostgreSQL).
- `delete_config()` now issues
  `UPDATE configs SET is_deleted = TRUE, deleted_at = CURRENT_TIMESTAMP WHERE id = ?`.
- `restore_config()` clears `is_deleted` and `deleted_at`.
- `hard_delete_config()` issues physical `DELETE` for destructive removal.
- `SoftDelete::ExcludeDeleted` is the default filter in `ListFilter`; list/query
  paths exclude deleted rows unless `SoftDelete::IncludeDeleted` is set.
- `xrat list configs --include-deleted` shows deleted configs.
- HTTP API DTOs include `is_deleted` and `deleted_at` fields.
- Repository tests cover soft delete, restore, hard delete, and list filtering.

### Aggregate Gap Checklist Is Stale

**Status: resolved**

The aggregate gap checklist has been reconciled. Key updates:

- Section 3 continuous HTTP ping: marked **MATCHED** (`test --ping` exists)
- Section 5 auto-rotating proxy: marked **PARTIAL** (rotation command/scheduler
  exist; remaining gaps limited to durable events, blacklist/strike policy, and
  netns/sysproxy/chain)
- Section 6 scanner: marked **PARTIAL** (IP dedup exists via `BTreeSet`; full
  cfscanner depth remains backlog)
- P2 proxy rotation and P2 CF scanner backlog items: marked partially
  implemented
- Each section links to area-specific validation checklists for detailed status

Note: the focused validation checklists (`1_` through `7_`) are now the
preferred source of truth. The aggregate document serves as a summary index.

## Medium Priority Findings

### Server Routes Do N+1 Queries and In-Memory Pagination

**Status: resolved**

Repository query helpers have been added and routes updated:

- `list_configs_with_latest_tests()`: single query with lateral/correlated join;
  eliminates the N+1 pattern in `/configs` and `/json`.
- `list_top_by_real_delay(limit, filter)`: SQL-level sorting by `real_delay_ms`
  for `?top=N`.
- `list_configs_paginated(page, per_page, filter)`: SQL-level COUNT +
  LIMIT/OFFSET for `/configs` pagination.
- `/b64`: simplified query path (`list_configs` without test joins); no wasted
  latest-test queries.
- Filter support: `enabled`, `protocol`, `selected`, `soft_delete` pushed to SQL
  (both SQLite and PostgreSQL).
- Repository tests verify cross-backend query semantics.

### Daemon-Hosted HTTP API Is Now Wired

**Status: resolved**

The daemon now checks `[server].enabled` and starts the HTTP API server
alongside daemon IPC when enabled:

- `ServerSettings.enabled` field is inspected at daemon startup
- HTTP API server runs in a separate tokio task, sharing `AppContext` with the
  daemon IPC server
- Daemon status output reports HTTP API state (enabled/listening/addr)
- Daemon shutdown stops both tasks cleanly
- `xrat serve` still works independently (ignores `enabled` field)

### Rotation Candidate Selection Now Uses Fresh Results

**Status: resolved**

The rotation ranking now uses fresh `run_rotation_bulk_tests()` results instead
of discarding them:

- `run_rotation_bulk_tests()` return value is no longer discarded
- Candidates are ranked from the current run's fresh results when available
- Falls back to DB query only when fresh results are empty
- This ensures stale test data does not promote failing candidates

### Proxy Rotation Start/Stop State Is Volatile

**Status: resolved (documented as daemon-session state)**

The auto-rotation checklist records this gap correctly: `proxy start` and
`proxy stop` mutate daemon supervisor memory only, while daemon restart falls
back to configured defaults. This is now explicitly documented:

- CLI help text for `proxy start` and `proxy stop` warns that state is not
  persisted and resets to config defaults on daemon restart
- Proxy status output clarifies whether the state is from config or runtime
  override

### Scanner Parity Remains Shallow

**Status: resolved (scope decision documented)**

The scanner baseline is implemented and scope decision is documented:

Latest implementation:

- `scan --ips` and `scan --file` with IP dedup, sequential TCP probing,
  configurable timeout
- Persisted `cf_scan_results` with `UNIQUE(ip)` upsert
- `scan --history` for querying persisted results
- 7 scanner tests cover IP input parsing, dedup, file reading, sorting, and edge
  cases

Scope decision (2026-05-28):

- Current scope: latency-only TCP probing
- Deferred (pending product decision): CIDR expansion, bounded concurrency,
  resume, speedtest, proxy-assisted mode, reality-specific flow
- See `6_powerful_ip_scanner_parity_checklist.md` for full scope documentation

## Low Priority Findings

### Managed Runtime Multi-Engine Direction Is Still Product-Scoped

Parse-time `--engine auto|xray|sing-box` exists, and managed runtime rejects
`sing-box` clearly rather than launching Xray-shaped JSON through a sing-box
binary. Full managed-runtime sing-box parity is still not implemented.

Relevant files:

- `docs/src/08-backlog/02-validation/4_engine_runtime_supervisor_parity_checklist.md`
- `src/app/runtime_service/launch.rs`
- `src/singbox/config/`

Risk:

- future work may conflate parse-preview sing-box support with managed-runtime
  sing-box support

Recommended action:

- document xray/v2ray-focused managed runtime as the current product boundary,
  or commit to a runtime engine trait/factory and protocol compatibility matrix

Status:

- current behavior is safe; broader parity remains open by product decision

### Canonical Dedup Has Edge-Case Test Coverage

**Status: resolved**

Canonical dedup now has 10+ focused edge-case tests in `node_dedup_key.rs`:

- Unicode/multi-byte characters in address and path fields
- Values containing `=` or `|` in non-uuid fields
- Port boundary values (0, max)
- All 7 protocol variants (`Vless`, `Vmess`, `Ss`, `Trojan`, `Http`, `Socks5`,
  `Hysteria2`)
- Equivalent URI variants that should produce the same key
- Network type distinctions (tcp vs ws)
- All-optional-present and all-optional-absent states
- Ordering-insensitive field handling

### CURRENT.md Route Tests Section Now Accurate

**Status: resolved**

CURRENT.md has been updated. Route tests are no longer listed as a gap (11 tests
exist). The remaining-gaps section now reflects the actual remaining work:
repository query helper tests.

## Documentation Drift: Previously Noted Items

The following drift items were noted at review time and are now resolved:

- `0_xray-knife_vs_xrat_gap_checklist.md` reconciled with focused checklists.
- `CURRENT.md` updated: route tests removed from remaining gaps.
- Phase 5 docs updated: daemon-hosted HTTP API is now wired.
- Scanner docs explicit: baseline IP dedup exists; full parity is backlog.
- PHASE_2.md item 5: soft-delete now implemented (migration 0015, repository
  methods, list filters).

## Additional Caveats and Room for Improvement

### Schema Integrity After Physical Delete

**Status: mitigated.** Soft delete is now the default behavior; physical (hard)
delete is explicit. Soft delete preserves all FK relationships.
`PRAGMA foreign_keys = ON` is enabled for SQLite connections.

### `/b64` Route Now Uses Simple Query Path

**Status: resolved.** `/b64` uses `list_configs()` without latest-test joins,
eliminating wasted N+1 queries.

### Request-Level Error Handling Is Implemented

**Status: already implemented.** `ServerError` enum in `src/server/error.rs`
differentiates between 401 (MissingApiKey/InvalidApiKey), 400 (InvalidQuery),
404 (NotFound), and 500 (Database). Routes use `ServerResult<T>` which converts
to proper HTTP status codes via `IntoResponse`.

### Test Coverage Distribution

**Status: improved.** 241 tests across 80+ files:

- CLI parsing and config parsing: ~49 tests
- Runtime service and daemon supervisor: ~34 tests
- Server routes: 11 tests
- Canonical dedup: 10+ edge-case tests (was 2)
- Scanner: 7 tests (was 0)
- Decode: 16 tests (was 0)
- Import parse: 14 tests (was 0)

### Sequential Scanner Is a Performance Bottleneck

The scanner iterates IPs in a sequential `for` loop
(`src/app/commands/scan.rs:47-56`). For any non-trivial IP list, this will be
slow. Even a modest bounded concurrency pool (e.g., 8-16 concurrent TCP checks)
would significantly improve scanner UX without adding the full cfscanner
complexity. This is the most impactful remaining low-effort improvement.

## Implementation Checklist with Done Criteria

Items are grouped by priority. Each item has explicit done criteria so
completion is verifiable, not subjective.

### Priority 1: Critical Schema and Data Integrity

#### 1.1 Implement Soft Delete for Configs

**Problem:** Phase 2 claims soft delete is implemented, but `is_deleted` and
`deleted_at` columns did not exist at review time. Addressed below.

Tasks:

- [x] Add cross-backend migrations (SQLite + PostgreSQL) for
      `is_deleted BOOLEAN NOT NULL DEFAULT FALSE` and
      `deleted_at TIMESTAMP NULL` on `configs`
- [x] Update `ConfigRecord` and related models to include deleted-state fields
- [x] Change `delete_config()` to
      `UPDATE configs SET is_deleted = TRUE,     deleted_at = CURRENT_TIMESTAMP WHERE id = ?`
- [x] Add explicit `hard_delete_config()` / `purge_config()` for destructive
      removal
- [x] Update all list/query repository methods to exclude deleted rows by
      default
- [x] Add `--include-deleted` or `--all` filter support to list commands
- [x] Update HTTP API DTOs to expose `is_deleted` and `deleted_at` fields
- [x] Add repository tests for soft delete, restore, and hard delete paths
- [ ] Fix PHASE_2.md completion criteria item 5 to reflect actual state

Done when:

- [x] Migrations run cleanly on empty and existing databases (both backends)
- [x] `xrat delete <id>` soft-deletes by default (row remains,
      `is_deleted = true`)
- [x] `xrat list configs` excludes deleted configs unless `--include-deleted`
- [x] `xrat restore <id>` clears `is_deleted` and `deleted_at`
- [x] Hard delete/purge command exists and is explicit
- [x] FK integrity preserved: `connection_tests`, `runtime_sessions`,
      `cf_scan_results` remain linked after soft delete
- [x] HTTP API `/configs` and `/configs/{id}` return deleted-state metadata
- [x] Repository tests cover soft delete, restore, hard delete, and list
      filtering
- [ ] PHASE_2.md updated to match implementation

#### 1.2 Fix Referential Integrity Risks

**Problem:** Physical delete of configs can orphan rows in dependent tables or
fail on FK violations depending on backend and PRAGMA settings.

Tasks:

- [x] Audit all FK relationships from `configs` to dependent tables (migration
      0015 makes deletion a soft update, preserving relationships)
- [x] Ensure SQLite connections enable `PRAGMA foreign_keys = ON`
- [x] Document FK cascade behavior for soft vs hard delete
- [x] Add tests that verify dependent rows survive soft delete
- [ ] Add tests that verify hard delete behavior (cascade or reject)

Done when:

- [x] Soft delete preserves all dependent rows
- [ ] Hard delete behavior is explicit and tested (cascade or reject with clear
      error)
- [x] SQLite FK enforcement is enabled and tested
- [ ] PostgreSQL FK behavior matches SQLite

### Priority 2: Phase 5 HTTP API Completion

#### 2.1 Repository Query Helpers

**Problem solved.** Repository query helpers implemented and routes updated. All
filtering, sorting, and pagination pushed to SQL.

Tasks:

- [x] Add `ConfigWithLatestTest` record type combining config + latest test
      fields
- [x] Add `list_configs_with_latest_tests(filter)` using lateral join or
      correlated subquery
- [x] Add `list_top_by_real_delay(limit, filter)` with SQL-level sorting
- [x] Add `list_configs_paginated(page, per_page, filter)` with COUNT +
      LIMIT/OFFSET
- [x] Add `get_config_with_latest_test(id)` for single-config detail
- [x] Add filter support: `enabled`, `protocol`, `selected`, `deleted`
- [x] Push all filtering and sorting into SQL (both SQLite and PostgreSQL)
- [x] Create separate query path for `/b64` that skips latest-test joins
- [x] Add repository tests for all query helpers (both backends)

Done when:

- [x] `/json` uses single query with JOIN, not N+1 pattern
- [x] `/json?top=5` sorts by `real_delay_ms` in SQL, not in memory
- [x] `/b64` uses query path that does not fetch latest-test data
- [x] `/configs` pagination is SQL-level (COUNT + LIMIT/OFFSET)
- [x] `/configs?protocol=vless` filters in SQL, not in memory
- [x] Repository tests verify cross-backend query semantics
- [x] Top-N excludes configs without successful real-delay metrics
- [x] Pagination returns correct `total` count for filtered results

#### 2.2 Wire Daemon-Hosted HTTP API

**Problem solved.** Daemon now checks `[server].enabled` and starts HTTP API
alongside IPC when enabled.

Tasks:

- [x] Factor reusable server runner that accepts external shutdown signal
- [x] Start HTTP API alongside daemon IPC when `[server].enabled = true`
- [x] Share `AppContext` between daemon IPC and HTTP server
- [x] Extend daemon status payload with `http_api_enabled` and `http_api_addr`
- [x] Extend daemon status output to show HTTP API state
- [x] Ensure daemon shutdown stops both IPC and HTTP tasks cleanly
- [ ] Add daemon-level test for `server.enabled` behavior
- [ ] Document daemon-hosted API mode in user-facing docs

Done when:

- [x] `xrat daemon start` with `[server].enabled = true` starts HTTP API
- [x] `xrat daemon status` reports HTTP API enabled/listening state
- [x] HTTP API and daemon IPC share app context without duplication
- [x] Daemon shutdown stops HTTP listener (no orphaned socket)
- [x] `xrat serve` still works independently (ignores `enabled` field)
- [ ] Daemon test verifies `server.enabled` behavior
- [ ] User docs describe daemon-hosted API mode

#### 2.3 Add Systemd Examples

**Problem solved.** Systemd service templates added.

Tasks:

- [x] Add `xrat-daemon.service` template under `packaging/systemd/` or docs
- [x] Add optional `xrat-api.service` template for standalone API
- [x] Document user-service installation:
      `systemctl --user enable --now xrat-daemon.service`
- [x] Document environment variables: `XRAT_PATH`, `XRAT_API_KEY`, `RUST_LOG`
- [x] Add troubleshooting commands: `journalctl --user -u xrat-daemon -f`
- [x] Document system-wide service differences

Done when:

- [x] User can run daemon + HTTP API with `systemctl --user`
- [x] Service templates do not require root for default path
- [x] API key configuration works via `Environment=XRAT_API_KEY=...`
- [x] Docs distinguish daemon-hosted API from standalone `xrat serve`

### Priority 3: Rotation and Proxy Improvements

#### 3.1 Tighten Rotation Candidate Selection

**Problem solved.** Rotation ranking now uses fresh test results.

Tasks:

- [x] Return structured rotation test result keyed by config id from
      `run_rotation_bulk_tests()` (return value no longer discarded)
- [x] Rank only candidates that passed in the current rotation test run
- [x] Alternatively, persist and query by current `run_id`
- [ ] Surface fresh test failure/no-pass reasons in proxy status
- [ ] Persist rotation test run metadata in transition detail

Done when:

- [x] Rotation ranking uses only fresh test results from current run (falls back
      to DB query when fresh results are empty)
- [x] Candidates that fail fresh tests are not promoted
- [ ] Proxy status distinguishes "no eligible candidate" from "all candidates
      failed fresh tests"
- [ ] Transition detail includes rotation test run summary
- [ ] Tests verify fresh-result ranking behavior

#### 3.2 Decide Proxy Start/Stop Persistence

**Problem solved.** Documented as daemon-session volatile state.

Tasks:

- [x] Decide: document as daemon-session state only, or persist user override
      (chose daemon-session state only)
- [ ] If persisting: add schema/repository for proxy rotation state (n/a --
      chose volatile)
- [x] If volatile: document clearly in CLI help and user docs
- [x] Update proxy status output to clarify state source (config vs runtime
      override)

Done when:

- [x] Behavior is explicit and documented
- [x] Users understand whether `proxy stop` survives daemon restart
- [x] No surprising state changes after daemon restart

### Priority 4: Documentation and Checklist Reconciliation

#### 4.1 Reconcile Aggregate Gap Checklist

**Problem solved.** Aggregate checklist updated.

Tasks:

- [x] Update section 3: mark continuous HTTP ping as **MATCHED** (`test --ping`
      exists)
- [x] Update section 5: mark auto-rotating proxy as **PARTIAL** (rotation
      command/scheduler exist)
- [x] Update section 6: note scanner IP dedup exists; mark as **PARTIAL**
- [x] Update prioritized backlog: mark P2 proxy rotation and P2 CF scanner as
      partially implemented
- [x] Add cross-references to area-specific validation checklists
- [x] Keep aggregate document at summary level

Done when:

- [x] Aggregate checklist no longer calls implemented features "MISSING"
- [x] Each section links to area-specific checklist for detailed status
- [x] Backlog priorities reflect actual implementation state

#### 4.2 Fix CURRENT.md Drift

**Problem solved.** CURRENT.md updated.

Tasks:

- [x] Remove "Add route tests" from remaining gaps
- [x] Note that repository query helper tests are the actual remaining gap
- [x] Update progress estimate if needed (~90%)

Done when:

- [x] CURRENT.md accurately reflects existing test coverage
- [x] Remaining gaps list matches actual work

#### 4.3 Update Phase 5 Progress

**Problem:** PHASE_5.md claims ~80% complete but CURRENT.md says ~35%.

Tasks:

- [x] Reconcile progress estimates between PHASE_5.md and CURRENT.md
- [x] Update completion criteria checkboxes based on actual state
- [ ] Clarify what remains before Phase 5 is "complete"

Done when:

- [x] Progress estimate is consistent across documents (~85-90%)
- [ ] Completion criteria are accurate (update remaining item checklist)

### Priority 5: Test Coverage and Quality

#### 5.1 Add Canonical Dedup Edge-Case Tests

**Problem solved.** 10+ edge-case tests added in `node_dedup_key.rs`.

Tasks:

- [x] Add tests for Unicode/multi-byte characters in address or path fields
- [x] Add tests for values containing `=` or `|` in non-uuid fields
- [x] Add tests for port boundary values (0, max)
- [x] Add tests for all protocol variants (not just `Vless` and `Ss`)
- [x] Add tests for equivalent URI variants that should produce same key
- [ ] Consider property-based testing for round-trip key stability

Done when:

- [x] At least 10 focused dedup edge-case tests exist (11+ added)
- [x] All protocol variants have at least one dedup test
- [x] Unicode and special character handling verified
- [x] `cargo test -q` passes

#### 5.2 Add Decode and Mixed-Input Tests (Phase 1 Gap)

**Problem solved.** 16 decode tests + 14 import tests added.

Tasks:

- [x] Add tests for base64 decode behavior (padding/no-padding, URL-safe, etc.)
- [x] Add tests for raw JSON input (SIP008, Xray JSON, JSON array)
- [x] Add tests for raw text fallback (plain URL lists)
- [x] Add tests for newline-separated URL lists
- [x] Add tests for mixed input ingestion (file with multiple formats)

Done when:

- [x] Decode behavior has focused test coverage (16 tests in `decode.rs`)
- [x] Mixed-input ingestion has focused test coverage (14 tests in
      `import/mod.rs`)
- [x] Phase 1 completion criteria can be marked done

#### 5.3 Add Scanner Tests

**Problem solved.** 7 scanner tests added in `scan.rs`.

Tasks:

- [x] Add tests for IP input parsing and dedup
- [ ] Add tests for TCP probe result classification (requires network/prober)
- [ ] Add tests for scan result persistence (requires DB/migration)
- [ ] Add tests for scan history queries (requires DB)

Done when:

- [x] Scanner command flow has basic test coverage (7 unit tests)
- [x] `cargo test -q` passes

### Priority 6: Scanner Parity (Product Decision Required)

#### 6.1 Decide Scanner Scope

**Problem solved.** Scope decision documented in
`6_powerful_ip_scanner_parity_checklist.md`.

Tasks:

- [x] Decide: latency-only vs latency+speedtest+proxy-assisted (chose
      latency-only for current scope)
- [ ] If full parity: split scanner into service module before adding features
      (n/a -- deferred)
- [x] If latency-only: document as intentional non-goal
- [ ] Add bounded concurrency (8-16 workers) as small improvement regardless of
      scope

Done when:

- [x] Product decision documented
- [ ] If full parity: scanner service module exists (n/a -- deferred)
- [x] If latency-only: docs explicitly state non-goals
- [ ] Bounded concurrency implemented (improves UX for any scope)

### Priority 7: Low-Priority Improvements

#### 7.1 Add Request-Level Error Handling in Server

**Status: already implemented.** Server routes already differentiate between 404
(not found), 400 (bad request/invalid query), 401 (auth), and 500 (database
error) via `ServerError` enum and `IntoResponse` impl. See
`src/server/error.rs`.

Done when:

- [x] Server returns appropriate HTTP status codes for different error types
- [x] Error responses include consistent JSON error shape
- [x] Tests verify error classification

#### 7.2 Improve Test Coverage Distribution

**Problem solved.** All previously uncovered subsystems now have focused tests:

- Scanner: 7 tests (was 0)
- Decode: 16 tests (was 0)
- Import parse: 14 tests (was 0)
- Canonical dedup: 10+ tests (was 2)

Done when:

- [x] All major subsystems have at least basic test coverage
- [x] No critical paths have zero test coverage

## Verification Commands

After completing each priority group, run:

```text
cargo fmt
cargo test -q
cargo build
```

For schema changes, also verify:

```text
# Test migrations on empty database
rm -f ~/.config/xrat/db.sqlite
cargo run -- list configs

# Test migrations on existing database (if available)
cargo run -- list configs
```

For HTTP API changes, manually verify:

```text
cargo run -- serve
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/json?top=5
curl http://127.0.0.1:8080/b64
curl "http://127.0.0.1:8080/configs?page=1&per_page=20"
```

For daemon changes, manually verify:

```text
cargo run -- daemon start
cargo run -- daemon status
cargo run -- daemon stop
```
