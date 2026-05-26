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
- Initial HTTP API implementation started:
  - `src/server/` module tree added
  - Axum router builder added
  - `GET /health`
  - `GET /json`
  - `GET /b64`
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

## Current Goal

Finish Phase 5 so XRAT can serve config data over HTTP in foreground mode and
daemon-hosted mode, with clear systemd user-service documentation.

Progress estimate: **~35%** complete.

## Remaining Gaps

1. Replace the current server-layer filtering with repository-level query
   helpers for config-with-latest-test, top-N real-delay sorting, and
   pagination.
2. Add route tests for `/health`, `/json`, `/b64`, `/configs`, and
   `/configs/{id}`.
3. Add daemon integration so `[server].enabled = true` starts the API alongside
   daemon IPC.
4. Extend daemon status output to report HTTP API enabled/listening state.
5. Add systemd user-service examples and docs for:
   - daemon-hosted API
   - optional standalone `xrat serve`
   - `XRAT_PATH`, `XRAT_API_KEY`, and `RUST_LOG`
6. Re-run broad verification in an environment that permits daemon sockets,
   ephemeral ports, and runtime process tests.

## Immediate Next Slice

1. Add route-level tests using the Axum router directly.
2. Move top-N and pagination behavior into DB/repository helpers.
3. Refactor server startup so `xrat serve` and daemon-hosted API share the same
   runner but use different shutdown signals.
4. Wire `[server].enabled = true` into daemon startup.
