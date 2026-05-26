# Current Implementation Review

Reviewed: 2026-05-26

Scope:

- `docs/src/02-backlog/01-plan/`
- `docs/src/02-backlog/02-validation/`
- current `src/` and migration implementation

Validation command:

```text
cargo test -q
```

Result: 184 tests passed after the first cleanup slice.

## Summary

The implementation is ahead of several checklist documents. Phases through 4.6
are mostly represented in code, and Phase 5 HTTP API code has already started,
but some planning and validation documents still describe older status.

The main risks are documentation drift, a soft-delete contract mismatch, runtime
engine ambiguity around `sing-box`, and Phase 5 server behavior that works but is
not yet shaped according to the planned repository/query/test boundaries.

## High Priority Findings

### Config Delete Is Physical, Not Soft Delete

Phase 2 documents expect configs to be marked deleted without physically removing
rows. The current schema has no `is_deleted` or `deleted_at` columns, and the
repository delete path physically deletes rows.

Relevant files:

- `migrations/sqlite/0001_init.sql`
- `migrations/postgres/0001_init.sql`
- `src/db/repository/configs/state_ops/mutations.rs`

Risk:

- runtime/history references can become harder to explain once delete/restore UX
  is exposed
- docs overpromise restore/soft-delete behavior that the current DB model cannot
  provide

Recommended action:

- either add soft-delete columns and repository behavior, or revise Phase 2 docs
  to state that physical delete is the current intentional behavior

### Managed Runtime `sing-box` Engine Is Misleading

The managed runtime can resolve `runtime.engine = "sing-box"` to the sing-box
binary path, but launch still generates Xray-shaped JSON.

Relevant files:

- `src/app/runtime_service/launch.rs`
- `src/app/config/proxy/types.rs`
- `src/singbox/config/`

Risk:

- users can configure `sing-box` and hit confusing runtime failures because the
  generated config is not a sing-box runtime config

Status:

- first cleanup slice now rejects `sing-box` for managed runtime with a clear
  error until a real runtime engine abstraction exists

### Phase 5 Planning Status Is Stale

`PHASE_5.md` says the phase has not started, while code already includes the
server module tree, `xrat serve`, server config, and route handlers.

Relevant files:

- `docs/src/02-backlog/01-plan/PHASE_5.md`
- `docs/src/02-backlog/01-plan/CURRENT.md`
- `src/server/`
- `src/cli/serve.rs`
- `src/app/commands/serve.rs`

Status:

- first cleanup slice updated Phase 5 planning status to reflect current code

Recommended action:

- continue updating detailed Phase 5 backlog checkboxes as repository helpers,
  route tests, and daemon-hosted HTTP land

## Medium Priority Findings

### Server Routes Do Query Work In Memory

The Phase 5 plan calls for repository helpers for latest-test joins, top-N real
delay sorting, and pagination. Current route handlers load configs, query latest
tests one by one, and filter/sort/page in memory.

Relevant files:

- `src/server/routes/json.rs`
- `src/server/routes/configs.rs`
- `src/db/repository/connection_tests/query_ops.rs`
- `src/db/repository/configs/import_ops/query.rs`

Risk:

- N+1 latest-test queries
- inefficient top-N and pagination for larger local databases
- backend-specific query behavior remains untested at the server boundary

Recommended action:

- add repository-level API query helpers for config-with-latest-test, top-N, and
  paginated listing across SQLite/PostgreSQL

### Daemon-Hosted HTTP API Is Not Wired

The standalone `xrat serve` path exists, but daemon startup does not start the
HTTP API when `[server].enabled = true`.

Relevant files:

- `src/app/commands/daemon.rs`
- `src/server/mod.rs`
- `src/app/config/server.rs`

Risk:

- one of the documented Phase 5 operating modes does not work yet

Recommended action:

- factor a reusable server runner with an external shutdown signal and start it
  alongside daemon IPC when configured

### Server Route Coverage Is Thin

Server auth unit tests exist, but route-level tests are missing for the main
endpoints.

Relevant files:

- `src/server/auth.rs`
- `src/server/routes/`

Recommended action:

- add router-level tests for `/health`, `/json`, `/b64`, `/configs`, and
  `/configs/{id}` including auth and error cases

### Rotation Candidate Selection Can Use Stale Data

Automatic rotation runs fresh bulk tests but ignores the returned result and then
ranks candidates from latest persisted test rows.

Relevant files:

- `src/app/runtime_service/replace_flow/candidate.rs`
- `src/app/commands/test/bulk/rotation.rs`

Risk:

- if fresh tests fail or partially fail, a candidate can still be promoted based
  on older persisted success data

Recommended action:

- make fresh rotation test failures visible in state/output and consider ranking
  only results from the current rotation run

### Proxy Rotation State Is Mostly Volatile

`proxy start` and `proxy stop` update daemon in-memory state. A daemon restart
falls back to config defaults.

Relevant files:

- `src/app/daemon/supervisor/handlers/runtime/runtime_lifecycle/proxy.rs`
- `src/app/daemon/supervisor/mod.rs`
- `src/app/config/proxy/types.rs`

Recommended action:

- document that `proxy start|stop` is daemon-session state, or persist the
  enabled/disabled override if users expect it to survive restart

### Scanner Parity Remains Shallow

The current scanner can probe explicit IPs and persist history, but it does not
yet match the broader cfscanner-style feature set.

Relevant files:

- `src/cli/scan.rs`
- `src/app/commands/scan.rs`
- `src/db/repository/cf_scan_results.rs`

Missing capabilities:

- CIDR/range expansion
- bounded concurrent worker pool
- resume semantics
- speedtest phase
- proxy-config-assisted scan
- reality-specific scanner flow

Recommended action:

- decide whether scanner parity is a product goal before expanding command UX

## Low Priority Findings

### HTTP Bind Address Parsing Is IPv4/Hostname Oriented

The server builds `host:port` as a string before parsing. IPv6 hosts such as
`::1` need bracket formatting.

Relevant file:

- `src/server/mod.rs`

Recommended action:

- parse host and port with IPv6-safe socket address construction

### `/b64` Exposes Raw Share Links By Design

`/b64` returns base64 subscription text from stored raw config links. This is
expected for subscription output, but it exposes credentials when the server is
bound to non-local interfaces without a key.

Relevant file:

- `src/server/routes/b64.rs`

Recommended action:

- make docs explicit that non-local binds should use `server.key`, and that
  query-string auth is only a v1 local/trusted-network mechanism

## Documentation Drift To Fix

- `docs/src/02-backlog/01-plan/PHASE_5.md` should no longer say 0% / not
  started.
- `docs/src/02-backlog/02-validation/5_auto_rotating_proxy_parity_checklist.md`
  should be updated to reflect Phase 4.6 implementation.
- `docs/src/02-backlog/02-validation/0_xray-knife_vs_xrat_gap_checklist.md`
  should be reconciled with the newer area-specific checklists.
- Phase 5/6 docs now record the product decision: support both soft delete and
  hard delete, with soft delete as the safe default and hard delete/purge as an
  explicit destructive action.

## Suggested Next Order

1. Fix documentation drift so the backlog is trustworthy.
2. Resolve the soft-delete contract mismatch.
3. Make managed-runtime engine behavior safe around `sing-box`.
4. Complete Phase 5 repository query helpers and route tests.
5. Wire daemon-hosted HTTP API.
6. Tighten rotation candidate selection around fresh test-run results.
