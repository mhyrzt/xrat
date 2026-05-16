# xrat Implementation Progress

> Last verified: 2026-05-16 · 177 tests passing · all phases through 4.6 implemented

## Completed Phases

| Phase | Area | Status |
|---|---|---|
| 1 | Subscription ingestion, parsing, normalization, dedup | **Complete** |
| 2 | SQLite + PostgreSQL persistence, migrations, repositories | **Complete** |
| 2.5 | CLI subcommand structure, command dispatch, thin main | **Complete** |
| 3 | Connection testing (ICMP/TCP/real-delay/download/upload), bulk, ping loop | **Complete** |
| 3.5 | Local app config (`config.toml`), routing, geo, DNS, testing defaults | **Complete** |
| 3.6 | Module refactors, tracing, canonical dedup key, PostgreSQL backend | **Complete** |
| 4 | Managed Xray runtime (connect/disconnect/status), stale reconciliation | **Complete** |
| 4.5 | Daemon supervisor, Unix IPC, reattach, replace, transition taxonomy | **Complete** |
| 4.6 | Auto-rotating proxy (start/status/rotate/stop), candidate scoring, triggers | **Complete** |

## Intentional Gaps

These are documented design decisions, not missing work.

### sing-box Runtime Process Spawn

- **What exists**: sing-box parse path for Hysteria2 (`src/singbox/config/`) generates valid runtime JSON for diagnostics (`parse --json --engine sing-box`).
- **What is missing**: No sing-box process spawn, no runtime lifecycle management for sing-box.
- **Why**: xrat runtime is xray-focused. sing-box support is parse-only until a product decision approves multi-engine runtime parity.
- **Files**: `src/singbox/config/mod.rs`, `src/singbox/config/hy2.rs`

### Multi-Engine Abstraction Trait

- **What exists**: Engine selection at parse time (`auto|xray|sing-box`) via `src/config/parse_service.rs`. Runtime binary path configurable via `[runtime].engine` (`xray|v2ray|sing-box`).
- **What is missing**: No runtime engine trait/factory abstraction equivalent to xray-knife `Core`. No protocol-to-engine compatibility matrix for managed runtime. No `auto` engine mode at runtime (only at parse time).
- **Why**: Current architecture keeps runtime orchestration xray-oriented. Multi-engine abstraction is deferred until sing-box runtime is approved.
- **Files**: `src/app/config/runtime/types.rs`, `src/config/parse_service.rs`

### Advanced Scanner Features

- **What exists**: `xrat scan` probes explicit IPs via TCP, persists results to `cf_scan_results`, supports `--history` queries.
- **What is missing**:
  - CIDR expansion / subnet scanning
  - Resume semantics (DB-backed or CSV-backed)
  - Integrated speedtest stage for top candidates
  - Proxy-config-assisted scan mode
  - Reality-specific scanner flow
  - CSV export contract
- **Why**: Scanner scope was limited to baseline TCP latency probing. Advanced features depend on product-direction confirmation on scanner depth.
- **Files**: `src/app/commands/scan.rs`, `src/cli/scan.rs`

### Continuous HTTP Ping (xray-knife Style)

- **What exists**: `xrat test <id> --ping` runs a continuous test loop with Ctrl+C summary, persisted under `ping_loop` run kind. Uses the full staged test pipeline (ICMP → TCP → real-delay → download).
- **What differs**: xray-knife `--ping` is a lightweight HTTP probe loop measuring ttfb/connect_ms/http_status per iteration. xrat's ping loop runs the full test pipeline per iteration, which is heavier but more comprehensive.
- **Why**: xrat reuses the existing test pipeline rather than implementing a separate lightweight HTTP probe path. Functionally equivalent for health monitoring, different performance profile.
- **Files**: `src/app/commands/test/entrypoints/ping.rs`

## Partial Implementations

### Scanner IP-Level Dedup

- **Status**: Missing (dependency on full scanner)
- **Why**: Scanner IP-level dedup map + DB unique key depends on full scanner subsystem (area #6). Not needed for current explicit-IP scan mode.
- **When**: Add when CIDR expansion and concurrent worker pool are implemented.

### Status Taxonomy Compatibility

- **Status**: Partial
- **What exists**: xrat uses simplified status model (`ok`, `failed`, `skipped`) with failure kind classification.
- **What differs**: xray-knife uses richer labels (`passed`, `semi-passed`, `timeout`, `broken`). No compatibility mapping table exists for cross-tool report parity.
- **Why**: xrat status model is intentionally simpler. Compatibility mapping is only needed if cross-tool report comparability becomes a hard requirement.
- **Files**: `src/app/commands/test/model/status.rs`

### Subscription CRUD/Fetch UX

- **Status**: Partial (intentional non-goal)
- **What exists**: Subscription source records stored on import, listable via `xrat list subscriptions`.
- **What is missing**: No `subs add/rm/update/fetch` command family like xray-knife. No `last_fetched_at` tracking or subscription refresh flow.
- **Why**: xrat keeps import-first UX to keep CLI surface small. Full subscription management is deferred.
- **Files**: `src/db/repository/subscriptions.rs`

## Out of Scope

These features were never planned for xrat.

| Feature | Reason |
|---|---|
| System proxy management (sysproxy) | Desktop OS-specific, not in scope |
| Network namespace orchestration (netns) | Linux-specific, not in scope |
| Multi-hop chain / outbound chaining | Not in xrat product scope |
| TUI application (Ratatui) | Planned for Phase 6, not yet started |
| HTTP API server (Axum) | Planned for Phase 5, not yet started |
| Full Xray JSON persistence | By design: generate runtime config on demand from stored normalized data |

## Deferred Phases

| Phase | Area | Status |
|---|---|---|
| 5 | HTTP API (`/json`, `/b64`, auth) | Not started |
| 6 | TUI application (Ratatui) | Not started |

## Test Coverage Summary

- **177 tests** passing across all modules
- Coverage includes: parser unit tests, dedup edge cases, CLI parsing, DB repository (SQLite + PostgreSQL), runtime service lifecycle, daemon IPC, reattach accept/reject, replace safety, rotation triggers, cooldown behavior, health tick suppression
