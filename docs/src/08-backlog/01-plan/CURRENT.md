# Current Work: Phase 5 HTTP API

## Context

Phases 1 through 4.6 are complete and provide the local workflow: import,
persistence, testing, managed runtime, daemon ownership, and auto-rotation.
Current implementation work is focused on Phase 5: an Axum HTTP API that exposes
stored configs, latest test data, and subscription-compatible output without
requiring direct database access.

The API should support three operating modes:

- foreground development via `xrat serve`
- daemon-hosted API via `[server].enabled = true`
- systemd/systemctl operation by running the daemon or standalone API as a user
  service

## What Landed So Far

- Phase 5 backlog expanded with detailed implementation slices:
  - server config
  - Axum router and routes
  - auth
  - response DTOs
  - config/test query support
  - `xrat serve`
  - daemon integration
  - systemd/systemctl examples
- mdBook summary now includes the Phase 5 page.
- HTTP API implementation:
  - `src/server/` module tree added
  - Axum router builder added
  - `GET /health`
  - `GET /json`
  - `GET /b64` (optimized to skip unnecessary test data joins)
  - `GET /configs`
  - `GET /configs/{id}`
  - JSON error response shape
  - optional `?key=` API-key enforcement
- Server config added to `AppConfig`:
  - `[server].enabled`
  - `[server].host`
  - `[server].port`
  - `[server].key`
- `xrat serve` CLI command added with `--host` and `--port` overrides.
- Example `[server]` block added to `testdata/config.example.toml`.
- Focused tests added for config parsing, CLI parsing, and auth behavior.
- Repository-level query helpers implemented:
  - `list_configs_with_latest_tests()` with JOIN
  - `list_top_configs_by_real_delay()` with SQL-level sorting
  - `list_configs_paginated_with_latest_tests()` with COUNT + LIMIT/OFFSET
  - `get_config_with_latest_test()` for single-config detail
  - `count_filtered()` for pagination totals
- Route-level tests added (11 tests covering routes and auth).
- Daemon integration wired: `[server].enabled = true` starts HTTP API alongside daemon IPC.
- Daemon status output reports `http_api_enabled` and `http_api_addr` fields.
- Systemd service examples added under `packaging/systemd/`.
- Soft delete implementation for configs with `is_deleted` and `deleted_at` columns.
- HTTP API DTOs expose `is_deleted` and `deleted_at` fields.

## Current Goal

Finish Phase 5 so XRAT can serve config data over HTTP in foreground mode and
daemon-hosted mode, with clear systemd user-service documentation.

Progress estimate: **~90%** complete.

## Remaining Gaps

1. Add systemd user-service examples and docs for:
   - daemon-hosted API
   - optional standalone `xrat serve`
   - `XRAT_PATH`, `XRAT_API_KEY`, and `RUST_LOG`
2. Re-run broad verification in an environment that permits daemon sockets,
   ephemeral ports, and runtime process tests.
3. Add soft-delete state to config records or explicitly gate deleted-state API
   fields until the Phase 6 config-management slice lands.

## Immediate Next Slice

1. Verify daemon-hosted serving in an environment that permits sockets, ephemeral
   ports, and runtime process tests.
2. Add repository-level tests for joined reads, top-N ordering, pagination, and
   protocol filtering.

