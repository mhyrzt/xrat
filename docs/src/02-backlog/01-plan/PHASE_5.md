# Phase 5 HTTP API

## Goal

Run a lightweight Axum-based HTTP server alongside the app to expose stored
configs and test results through simple REST-style endpoints. The server enables
external clients (browsers, other tools, or subscription consumers) to consume
xrat-managed config data without direct database access.

By the end of this phase, XRAT should be able to:

- start an HTTP listener on a configurable host and port
- return configs as JSON through `/json` and `/configs`
- return configs as base64-encoded subscription text through `/b64`
- filter and sort results by real-delay test metrics using a `top` query
  parameter
- protect endpoints with an optional API key via `?key=` query parameter
- expose a lightweight `/health` check with no auth required
- serve the server standalone via `xrat serve` or alongside the daemon when
  enabled

## Why This Phase Exists

Phases 1 through 4 give XRAT a complete local workflow:

- import and store configs from subscriptions
- test connectivity and measure real delay
- run a managed Xray runtime session

But all of that data lives behind the CLI and the local database.

Phase 5 exists to answer a different access pattern:

- can other tools or clients consume xrat's config data over HTTP?
- can xrat act as a local subscription server that other proxy clients can
  subscribe to?
- can scripts and automation query config state without parsing CLI output?

This phase turns xrat from a standalone CLI tool into a local data provider.

## Current Starting Point

Phase 5 should explicitly build on the primitives and data-model work completed
through Phase 4.

Those reusable pieces include:

- `ConfigRecord` and `ConfigListFilter` in `src/db/record/configs.rs`
- `ConnectionTestRecord` and latest-test queries in
  `src/db/repository/connection_tests/`
- `node_from_record()` helper to reconstruct `Node` from DB rows
- `generate_runtime_config_for_inbounds()` in `src/xray/config/generator/`
- `AppContext` with `DbPool`, `AppConfig`, and `RuntimePaths`
- `SecretString` pattern for configurable secrets (literals or env vars)
- `[runtime.socks]`, `[runtime.http]`, `[runtime.shadowsocks]` config sections
- `RuntimeService` with connect/disconnect/status lifecycle
- concrete `AppError`/`DbError` types instead of boxed errors
- tracing-based diagnostics throughout the app
- SQLite and PostgreSQL dual-backend support in the repository layer
- canonical `configs.dedup_key` for stable node identity
- daemon IPC framework in `src/app/daemon/` for potential server co-hosting

Phase 5 reuses these for HTTP serving instead of CLI-only access.

## Scope Boundary

Phase 5 should cover:

- `src/server/` module tree with Axum routes, handlers, and state
- core endpoints: `/health`, `/json`, `/b64`, `/configs`, `/configs/:id`
- `top` query parameter for real-delay sorting on `/json` and `/b64`
- optional `key` query parameter for request authentication
- `[server]` config section with `enabled`, `host`, `port`, and `key`
- `xrat serve` CLI command to start the server standalone
- DB query helpers for config-with-test joins and paginated listing
- focused tests for route handlers, auth middleware, and query helpers

Phase 5 should not yet cover:

- TLS termination (defer until auth moves beyond a simple query key)
- full CRUD over HTTP (configs are managed through CLI/daemon commands)
- WebSocket or streaming endpoints
- admin dashboard or HTML views
- config mutation endpoints (POST/PUT/DELETE)
- user management or role-based access control
- rate limiting or request throttling
- Prometheus metrics or structured request logging
- the TUI application (Phase 6)

Those belong to later phases or future iterations.

Deletion policy note for Phase 5/6:

- XRAT should support both soft delete and hard delete for stored configs.
- Soft delete is the safer default for user-facing flows because it preserves
  runtime history, connection-test history, and recovery options.
- Hard delete should be an explicit destructive action selected by the user, not
  an accidental default.
- Phase 5 HTTP API should expose deleted-state metadata when returning config
  detail/list data once the schema supports it, but should not add mutation
  endpoints in v1.
- Phase 6 TUI should make the distinction clear in the UI, for example:
  `Delete` = soft delete, `Purge` or `Hard delete` = permanent removal with
  confirmation.

## Desired User Experience

The first usable version should feel like this:

- `xrat serve`
  - starts HTTP server on configured host/port
  - logs bind address and readiness
  - blocks until SIGINT/SIGTERM

- `curl http://127.0.0.1:8080/health`
  - returns `{"status":"ok"}`

- `curl http://127.0.0.1:8080/json?top=5`
  - returns the 5 fastest configs by real delay as JSON

- `curl http://127.0.0.1:8080/b64`
  - returns base64-encoded subscription text usable by other clients

- `curl http://127.0.0.1:8080/configs?page=1&per_page=20`
  - returns paginated config list with full metadata

- `curl http://127.0.0.1:8080/configs/12`
  - returns config 12 with its latest test result

- when `server.key` is configured:
  - `curl http://127.0.0.1:8080/json` → `401 Unauthorized`
  - `curl http://127.0.0.1:8080/json?key=correct` → `200 OK`

Later extensions can add:

- `xrat serve --background` (daemon-hosted server)
- `Authorization: Bearer` header support alongside `?key=`
- `/configs/:id/test` for full test history
- `/subscriptions` endpoint for source metadata
- CORS configuration for browser-based clients

## Configuration

Add a `[server]` section to `config.toml`:

```toml
[server]
enabled = false
host = "127.0.0.1"
port = 8080
key = { env = "XRAT_API_KEY" }
```

- `enabled`: controls whether the server starts automatically with the daemon.
  It does not affect explicit `xrat serve`; that command starts the server even
  when `enabled = false`.
- `host` / `port`: bind address for the Axum listener.
- `key`: optional `SecretString`. When configured, all requests except `/health`
  must include `?key=<value>` or receive `401`.

Recommended operating modes:

1. Foreground development:

   ```bash
   xrat serve
   xrat serve --host 127.0.0.1 --port 9090
   ```

2. Daemon-managed API:

   ```toml
   [server]
   enabled = true
   host = "127.0.0.1"
   port = 8080
   key = { env = "XRAT_API_KEY" }
   ```

   Then start the daemon through the existing daemon lifecycle:

   ```bash
   xrat daemon start
   xrat daemon status
   xrat daemon stop
   ```

3. Systemd-managed service:

   Run either the daemon, which can host runtime IPC plus the HTTP API, or the
   standalone HTTP server as a dedicated unit:

   ```bash
   systemctl --user enable --now xrat-daemon.service
   systemctl --user status xrat-daemon.service
   ```

   For system-wide installs, use `sudo systemctl ...` and a system service file.
   The first implementation should document user services first because they do
   not require root or privileged install paths.

## Endpoints

### `GET /health`

Lightweight reachability check. No auth required.

Response:

```json
{ "status": "ok" }
```

### `GET /json`

Return configs as a JSON array. Supports filtering and sorting.

Query parameters:

| Param      | Type    | Required | Description                                      |
| ---------- | ------- | -------- | ------------------------------------------------ |
| `key`      | string  | No       | API key (required if `server.key` is configured) |
| `top`      | integer | No       | Return only the top N configs by real delay      |
| `enabled`  | bool    | No       | Filter to only enabled configs (default: true)   |
| `protocol` | string  | No       | Filter by protocol (e.g., `vless`, `trojan`)     |

Response shape (array of):

```json
{
  "id": 1,
  "name": "US Node 1",
  "protocol": "vless",
  "address": "1.2.3.4",
  "port": 443,
  "network": "ws",
  "tls": "tls",
  "real_delay_ms": 120,
  "tcp_ok": true,
  "last_tested_at": "2026-05-20T12:00:00Z"
}
```

- `real_delay_ms`, `tcp_ok`, `last_tested_at` come from the latest
  `connection_tests` row for each config. They are `null` when no test exists.
- When `top` is specified, only configs with a valid `real_delay_ms` are
  considered, sorted ascending (lowest delay = best).

### `GET /b64`

Return configs as base64-encoded subscription text (one URI per line, then
base64). Same query parameters as `/json`.

Response:

```
<base64-encoded string of raw_config lines joined by newlines>
```

- Content-Type: `text/plain`
- Only `enabled` configs are included by default.
- When `top` is specified, the same real-delay sort applies before encoding.

### `GET /configs`

Return a paginated list of all stored configs with full metadata.

Query parameters:

| Param      | Type    | Required | Description                            |
| ---------- | ------- | -------- | -------------------------------------- |
| `key`      | string  | No       | API key                                |
| `page`     | integer | No       | Page number (1-based, default: 1)      |
| `per_page` | integer | No       | Items per page (default: 50, max: 200) |
| `enabled`  | bool    | No       | Filter to only enabled configs         |
| `protocol` | string  | No       | Filter by protocol                     |
| `deleted`  | bool    | No       | Include or filter soft-deleted configs once supported |

Response:

```json
{
  "total": 150,
  "page": 1,
  "per_page": 50,
  "items": [
    {
      "id": 1,
      "subscription_id": 1,
      "dedup_key": "...",
      "protocol": "vless",
      "address": "1.2.3.4",
      "port": 443,
      "name": "US Node 1",
      "network": "ws",
      "tls": "tls",
      "is_active": false,
      "is_enabled": true,
      "is_selected": false,
      "is_deleted": false,
      "imported_at": "...",
      "created_at": "...",
      "updated_at": "..."
    }
  ]
}
```

### `GET /configs/:id`

Return a single config by ID with its latest test result.

Query parameters:

| Param | Type   | Required | Description |
| ----- | ------ | -------- | ----------- |
| `key` | string | No       | API key     |

Response:

```json
{
  "id": 1,
  "subscription_id": 1,
  "dedup_key": "...",
  "protocol": "vless",
  "address": "1.2.3.4",
  "port": 443,
  "name": "US Node 1",
  "network": "ws",
  "tls": "tls",
  "is_active": false,
  "is_enabled": true,
  "is_selected": false,
  "is_deleted": false,
  "imported_at": "...",
  "created_at": "...",
  "updated_at": "...",
  "latest_test": {
    "tcp_ok": true,
    "tcp_ms": 95,
    "real_delay_ok": true,
    "real_delay_ms": 120,
    "tested_at": "..."
  }
}
```

Returns `404` if the config does not exist.

## Auth Middleware

When `server.key` is configured, all endpoints except `/health` require the
`key` query parameter to match the resolved secret value.

- Missing key → `401 Unauthorized` with `{"error": "missing api key"}`
- Wrong key → `401 Unauthorized` with `{"error": "invalid api key"}`

Implementation: an Axum middleware or extractor that checks the query param
against the resolved `SecretString` from config.

## Module Structure

```
src/server/
  mod.rs            # Router builder, server entry point, shutdown handling
  state.rs          # ServerState (DbPool, optional resolved api key)
  auth.rs           # ApiKey extractor / middleware
  routes/
    mod.rs          # Route aggregation
    health.rs       # GET /health
    json.rs         # GET /json
    b64.rs          # GET /b64
    configs.rs      # GET /configs, GET /configs/:id
  response.rs       # Shared response types (JsonApiResponse, PaginatedResponse)
  error.rs          # Server-specific error types (IntoResponse impls)
```

## Database Queries Needed

Add new query helpers under `src/db/repository/configs/` or a new
`src/db/repository/server/` module:

- `list_top_by_real_delay(pool, limit, filter) → Vec<ConfigWithTest>`
  - JOIN `configs` with latest `connection_tests` per config
  - ORDER BY `real_delay_ms ASC`
  - WHERE `real_delay_ms IS NOT NULL`
- `list_configs_paginated(pool, page, per_page, filter) → (total, Vec<ConfigRecord>)`
- `get_config_with_latest_test(pool, id) → Option<ConfigWithTest>`

A `ConfigWithTest` struct should combine `ConfigRecord` fields with the latest
`ConnectionTestRecord` fields needed for API responses.

## CLI Integration

Add a `serve` command to start the HTTP server standalone:

```
xrat serve              # start server with config defaults
xrat serve --port 9090  # override port
xrat serve --host 0.0.0.0  # override host
```

CLI args (`src/cli/serve.rs`):

```rust
#[derive(Debug, Args)]
pub struct ServeArgs {
    #[arg(long, help = "Override server bind host")]
    host: Option<String>,
    #[arg(long, help = "Override server bind port")]
    port: Option<u16>,
}
```

The server blocks on `tokio::signal::ctrl_c()` for clean shutdown.

## Detailed Implementation Plan

### Step 1. Add server config section

Add `ServerSettings` to `AppConfig` under `src/app/config/`:

```rust
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ServerSettings {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub key: Option<SecretString>,
}
```

Wire defaults and add to `AppConfig` struct.

### Step 2. Create `src/server/` module skeleton

Set up the module tree with `mod.rs`, `state.rs`, `auth.rs`, `routes/`,
`response.rs`, and `error.rs`. Build a minimal router with a single `/health`
route to verify Axum integration compiles.

### Step 3. Implement `/health`

Return `{"status": "ok"}` with no auth, no DB dependency. Verify with a quick
test using `axum::Router` and `http::Request`.

### Step 4. Implement auth middleware

Create an Axum extractor or middleware layer that:

- reads the `key` query parameter
- compares it against the resolved `SecretString` from `ServerSettings`
- returns `401` on missing or mismatch
- passes through on match or when no key is configured

Apply to all routes except `/health`.

### Step 5. Add DB query helpers

Create the new repository functions needed by the API:

- `ConfigWithTest` struct combining config + latest test fields
- `list_top_by_real_delay()` with JOIN and ORDER BY
- `list_configs_paginated()` with COUNT + LIMIT/OFFSET
- `get_config_with_latest_test()` for single-config detail

Support both SQLite and PostgreSQL syntax via the existing `DbPool` pattern.

### Step 6. Implement `/json`

Build the handler that:

- parses query parameters (`top`, `enabled`, `protocol`)
- calls the appropriate DB query helper
- serializes results to the response shape
- handles empty results gracefully (empty array, not null)

### Step 7. Implement `/b64`

Build the handler that:

- reuses the same query logic as `/json`
- extracts `raw_config` from each result
- joins with newlines, base64-encodes
- returns `text/plain` content type

### Step 8. Implement `/configs` with pagination

Build the handler that:

- parses `page`, `per_page`, `enabled`, `protocol`
- validates `per_page` bounds (default 50, max 200)
- calls paginated query helper
- returns `{ total, page, per_page, items }` shape

### Step 9. Implement `/configs/:id`

Build the handler that:

- extracts `id` from path
- calls `get_config_with_latest_test()`
- returns `404` if not found
- includes `latest_test` field (nullable) in response

### Step 10. Add `serve` CLI command

Add `ServeArgs` to `src/cli/`, wire into `Command` enum, and create the command
handler in `src/app/commands/serve.rs`. The handler should:

- build `AppContext`
- resolve server settings (with CLI overrides)
- construct the Axum router with `ServerState`
- bind and listen
- wait for SIGINT/SIGTERM
- shut down gracefully

### Step 11. Add tests

Cover:

- route handler responses (happy path and edge cases)
- auth middleware (missing key, wrong key, no key configured, correct key)
- DB query helpers (top N, pagination, config-with-test join)
- CLI parsing for `serve` command
- graceful shutdown behavior

### Step 12. Wire into daemon (optional)

If `server.enabled = true`, start the HTTP server alongside the IPC socket in
`daemon run-server`. This can be deferred to a follow-up iteration if the
standalone `xrat serve` command is sufficient for the first release.

## Recommended Delivery Order

To keep risk low, build this phase in the following order:

1. add `[server]` config section and `ServerSettings` struct
2. create `src/server/` module skeleton with minimal `/health` route
3. implement auth middleware / key extractor
4. add DB query helpers (`ConfigWithTest`, top-N, pagination, config-with-test)
5. ship `/json` with `top`, `enabled`, `protocol` filters
6. ship `/b64` with same filters and base64 encoding
7. ship `/configs` with pagination
8. ship `/configs/:id` with latest test join
9. add `xrat serve` CLI command with signal-based shutdown
10. add focused route, auth, query, and CLI tests
11. (optional) wire server into daemon when `server.enabled = true`

## Detailed Backlog

This backlog is intentionally ordered so each slice compiles and can be tested
before moving to the next one. Keep route logic thin: handlers should parse
HTTP input, call repository helpers, and map domain records into response DTOs.

### P5.1 Server configuration

Goal: make HTTP server settings part of normal app configuration without
changing runtime behavior yet.

Tasks:

- [ ] Add server defaults in `src/app/config/defaults.rs`:
  - `DEFAULT_SERVER_ENABLED = false`
  - `DEFAULT_SERVER_HOST = "127.0.0.1"`
  - `DEFAULT_SERVER_PORT = 8080`
- [ ] Add `ServerSettings` under `src/app/config/` with:
  - `enabled: bool`
  - `host: String`
  - `port: u16`
  - `key: Option<SecretString>`
- [ ] Add `server: ServerSettings` to `AppConfig`.
- [ ] Add `Default` implementation and serde defaults.
- [ ] Update `testdata/config.example.toml` with commented `[server]`
      examples.
- [ ] Add config tests for empty config defaults, literal key, env key, and
      host/port override.

Acceptance:

- [ ] Existing configs still deserialize with defaults.
- [ ] Missing `[server]` section does not change app behavior.
- [ ] `SecretString` resolution behavior is reused, not duplicated.

### P5.2 Server module skeleton

Goal: introduce the Axum boundary and a health route without database coupling.

Tasks:

- [ ] Create `src/server/mod.rs`.
- [ ] Create `src/server/state.rs` with `ServerState`.
- [ ] Create `src/server/routes/mod.rs`.
- [ ] Create `src/server/routes/health.rs`.
- [ ] Create `src/server/error.rs` with a server error type and `IntoResponse`.
- [ ] Create `src/server/response.rs` for shared response DTOs.
- [ ] Add a router builder function, for example `build_router(state)`.
- [ ] Add a `/health` route returning `{"status":"ok"}`.
- [ ] Wire `mod server;` in `src/main.rs` or crate root as needed.

Acceptance:

- [ ] `cargo build` succeeds with the new module tree.
- [ ] A route-level test can call `/health` without a live TCP listener.
- [ ] `/health` has no DB dependency and no auth dependency.

### P5.3 API state and auth

Goal: support optional API key enforcement for all non-health routes.

Tasks:

- [ ] Decide whether auth is middleware or extractor; prefer extractor if it
      keeps per-route behavior explicit and easy to test.
- [ ] Resolve `server.key` once when building `ServerState`.
- [ ] Store only the resolved value needed for comparison in `ServerState`.
- [ ] Parse `key` from query string.
- [ ] Return `401` with `{"error":"missing api key"}` when a key is required
      but absent.
- [ ] Return `401` with `{"error":"invalid api key"}` when a key mismatches.
- [ ] Allow requests when no server key is configured.
- [ ] Ensure `/health` remains unauthenticated.

Acceptance:

- [ ] Tests cover no configured key, missing key, wrong key, correct key.
- [ ] Auth behavior is independent of route internals.
- [ ] Error responses use one consistent JSON error shape.

### P5.4 API response models

Goal: define stable v1 JSON shapes before writing route handlers.

Tasks:

- [ ] Add response DTOs for:
  - `HealthResponse`
  - `ApiConfigSummary`
  - `ApiConfigDetail`
  - `ApiLatestTest`
  - `PaginatedResponse<T>`
  - `ApiErrorResponse`
- [ ] Keep DTOs independent from SQLx row structs.
- [ ] Map protocol, address, port, name, network, TLS,
      active/enabled/selected/deleted, import timestamps, and latest-test fields
      explicitly once soft-delete fields exist.
- [ ] Decide nullability for latest-test fields and document it in code tests.
- [ ] Add serde tests or route tests that assert representative JSON keys.

Acceptance:

- [ ] Handlers can serialize without exposing internal DB-only fields unless
      intentionally included.
- [ ] `latest_test` is `null` when no test exists.
- [ ] `/json` response stays compact; `/configs` and `/configs/:id` carry the
      richer metadata.

### P5.5 Repository query helpers

Goal: provide server-ready data access while preserving SQLite/PostgreSQL
support.

Tasks:

- [ ] Add `ConfigWithLatestTest` or `ConfigWithTest` record type.
- [ ] Add a query helper for latest test per config.
- [ ] Add `list_api_configs(filter)` for `/json` and `/b64`.
- [ ] Add `list_top_by_real_delay(limit, filter)`.
- [ ] Add `list_configs_paginated(page, per_page, filter)`.
- [ ] Add `get_config_with_latest_test(id)`.
- [ ] Add filter support for:
  - enabled status
  - protocol
  - selected status if trivial with existing fields
  - deleted status once soft-delete fields exist
- [ ] Use the existing DB backend pattern for SQLite and PostgreSQL query
      differences.
- [ ] Clamp pagination in application code before hitting the DB.

Acceptance:

- [ ] Top-N excludes configs with no successful real-delay metric.
- [ ] Top-N sorts ascending by real-delay milliseconds.
- [ ] Pagination returns `(total, items)` consistently.
- [ ] Query tests cover SQLite; PostgreSQL SQL should follow the existing
      repository pattern even if CI only runs SQLite.

### P5.6 Query parsing and validation

Goal: keep route handlers deterministic and defensive.

Tasks:

- [ ] Add query structs for:
  - `/json` and `/b64`: `top`, `enabled`, `protocol`, `selected`, `deleted`,
    `key`
  - `/configs`: `page`, `per_page`, `enabled`, `protocol`, `selected`,
    `deleted`, `key`
- [ ] Default `enabled` to `true` for `/json` and `/b64`.
- [ ] Default `page` to `1`.
- [ ] Default `per_page` to `50`.
- [ ] Clamp or reject `per_page > 200`; prefer rejecting with `400` so clients
      notice invalid input.
- [ ] Reject `top=0` and unreasonably large `top`; suggested max is `200`.
- [ ] Return `400` with JSON error for invalid query values.

Acceptance:

- [ ] Invalid pagination and top values do not hit the repository layer.
- [ ] Empty result sets return empty collections, not errors.
- [ ] Defaults match the endpoint documentation.

### P5.7 `GET /json`

Goal: expose subscription-consumable config summaries as JSON.

Tasks:

- [ ] Add `src/server/routes/json.rs`.
- [ ] Apply auth to the route.
- [ ] Parse shared config-list query.
- [ ] Use top-N query when `top` is present.
- [ ] Otherwise list configs using the default filter.
- [ ] Map records to `ApiConfigSummary`.
- [ ] Return an array, not an envelope, for compatibility with simple clients.
- [ ] Add route tests for default list, `top`, protocol filter, and auth.

Acceptance:

- [ ] `GET /json` returns only enabled configs by default.
- [ ] `GET /json?enabled=false` can return disabled configs if supported by
      the filter contract.
- [ ] `GET /json?top=5` returns at most five configs sorted by real delay.
- [ ] No raw secrets are included in JSON output.

### P5.8 `GET /b64`

Goal: let XRAT act as a local subscription source.

Tasks:

- [ ] Add `src/server/routes/b64.rs`.
- [ ] Reuse the same query parsing and repository selection as `/json`.
- [ ] Extract the raw share URI or reconstruct a share URI from stored node
      fields if raw config is unavailable.
- [ ] Join URIs with newline.
- [ ] Base64-encode using the existing `base64` dependency.
- [ ] Return `Content-Type: text/plain; charset=utf-8`.
- [ ] Return an empty text body for empty results.
- [ ] Add tests for default output, top-N output, and auth.

Acceptance:

- [ ] Decoded response is one URI per line.
- [ ] Output ordering matches `/json` for the same filter.
- [ ] The route does not return JSON envelopes or quoted strings.

### P5.9 `GET /configs`

Goal: provide paginated management-oriented config metadata.

Tasks:

- [ ] Add list handler in `src/server/routes/configs.rs`.
- [ ] Apply auth to the route.
- [ ] Parse pagination and filters.
- [ ] Call `list_configs_paginated`.
- [ ] Return `{ total, page, per_page, items }`.
- [ ] Include stable identity fields such as `id`, `subscription_id`, and
      `dedup_key`.
- [ ] Include status fields such as `is_active`, `is_enabled`, and
      `is_selected`.
- [ ] Add tests for default pagination, custom pagination, max page size, and
      empty pages.

Acceptance:

- [ ] `page` is one-based.
- [ ] `total` reflects the filtered total, not just the current page count.
- [ ] `per_page` never exceeds the documented max.

### P5.10 `GET /configs/:id`

Goal: provide one detailed record with latest test context.

Tasks:

- [ ] Add detail handler in `src/server/routes/configs.rs`.
- [ ] Apply auth to the route.
- [ ] Parse `id` as the path parameter.
- [ ] Call `get_config_with_latest_test`.
- [ ] Return `404` with JSON error when missing.
- [ ] Include `latest_test: null` if the config exists but has no tests.
- [ ] Add tests for found, missing, no latest test, and auth.

Acceptance:

- [ ] Missing config IDs do not leak SQL errors.
- [ ] Detail response includes the same core config fields as `/configs`.
- [ ] Latest-test fields align with persisted connection-test terminology.

### P5.11 `xrat serve` CLI

Goal: start the HTTP API as a standalone foreground process.

Tasks:

- [ ] Add `src/cli/serve.rs` with `ServeArgs`.
- [ ] Add `Serve` variant to the root command enum.
- [ ] Add CLI parsing tests for `serve`, `serve --host`, and `serve --port`.
- [ ] Add `src/app/commands/serve.rs`.
- [ ] Resolve app context and server settings.
- [ ] Apply CLI host/port overrides after config loading.
- [ ] Bind with `tokio::net::TcpListener`.
- [ ] Start Axum serving with graceful shutdown.
- [ ] Log bind address and shutdown events through tracing.
- [ ] Return an app error if the bind address is unavailable.

Acceptance:

- [ ] `xrat serve --host 127.0.0.1 --port 9090` starts on that address.
- [ ] Ctrl+C stops the server cleanly.
- [ ] Bind failures surface as actionable CLI errors.

### P5.12 Daemon integration

Goal: optionally run the API alongside the daemon without coupling HTTP
lifecycle to IPC internals.

Tasks:

- [ ] Include this in Phase 5 initial delivery. `server.enabled = true` is the
      supported long-running API mode for users who already run `xrat daemon`.
- [ ] Read `server.enabled`, `server.host`, `server.port`, and `server.key`
      during daemon startup.
- [ ] Keep `xrat serve` independent: explicit foreground serving ignores
      `server.enabled` and only uses `host`, `port`, and `key`.
- [ ] Factor the server runner so daemon mode can start it with a shutdown
      signal instead of waiting directly on Ctrl+C.
- [ ] Spawn the HTTP server task separately from the IPC listener.
- [ ] Share app context safely without duplicating DB pools unnecessarily.
- [ ] Ensure daemon shutdown stops both IPC and HTTP tasks through one
      cancellation path.
- [ ] Log HTTP startup failure and fail daemon startup rather than silently
      running without the requested server.
- [ ] Report API bind address in daemon status output when enabled.
- [ ] Make bind conflicts actionable: include `host:port` and likely cause in
      the error.
- [ ] Add a daemon lifecycle test if practical.

Acceptance:

- [ ] `server.enabled = false` keeps daemon behavior unchanged.
- [ ] `server.enabled = true` starts HTTP and IPC together.
- [ ] Daemon shutdown does not leave the HTTP listener running.
- [ ] `xrat daemon status` shows whether the HTTP API is enabled and listening.
- [ ] `xrat serve` and daemon-hosted API use the same router and response
      behavior.

### P5.12a Systemd and systemctl integration

Goal: make XRAT installable as a long-running service without inventing a
second daemon model.

Tasks:

- [ ] Decide service shape:
  - primary: `xrat-daemon.service` runs `xrat daemon serve` or the internal
    foreground daemon command used by `xrat daemon start`
  - optional: `xrat-api.service` runs `xrat serve` for users who only want the
    HTTP subscription/API server
- [ ] Add service templates under `packaging/systemd/` or
      `docs/src/02-backlog/01-plan/examples/systemd/`:
  - `xrat-daemon.service`
  - optional `xrat-api.service`
- [ ] Prefer user-service examples first:
  - `~/.config/systemd/user/xrat-daemon.service`
  - `systemctl --user enable --now xrat-daemon.service`
- [ ] Document system-wide service differences:
  - install path for the binary
  - service user
  - writable XRAT state/config directory
  - environment file location
- [ ] Support environment variables in unit examples:
  - `XRAT_PATH`
  - `XRAT_API_KEY`
  - optional `RUST_LOG`
- [ ] Document the recommended config for daemon-hosted HTTP:
  - `[server].enabled = true`
  - `[server].host = "127.0.0.1"` by default
  - `[server].port = 8080`
  - `[server].key = { env = "XRAT_API_KEY" }`
- [ ] Add `ExecStart` examples that do not require shell wrapping.
- [ ] Add `Restart=on-failure` and a conservative restart delay.
- [ ] Add troubleshooting commands:
  - `systemctl --user status xrat-daemon`
  - `journalctl --user -u xrat-daemon -f`
  - `curl http://127.0.0.1:8080/health`
- [ ] Keep systemd packaging optional; the core app must still work without
      systemd.

Acceptance:

- [ ] A user can run XRAT daemon plus HTTP API with `systemctl --user`.
- [ ] The unit examples do not require root for the default documented path.
- [ ] API key configuration works through `Environment=XRAT_API_KEY=...` or an
      `EnvironmentFile=`.
- [ ] Docs clearly distinguish daemon-hosted API from standalone `xrat serve`.

### P5.13 Documentation and examples

Goal: make the new API usable without reading the implementation.

Tasks:

- [ ] Add example config block for `[server]`.
- [ ] Document the difference between:
  - `xrat serve` foreground server
  - daemon-hosted API via `[server].enabled = true`
  - systemd-managed daemon via `systemctl --user`
- [ ] Add curl examples for health, JSON, b64, config list, and config detail.
- [ ] Document default filters and pagination bounds.
- [ ] Document auth behavior and the local/trusted-network security assumption.
- [ ] Document config deletion semantics: soft delete is the safe default; hard
      delete/purge is explicit and destructive.
- [ ] Add a decoded `/b64` example showing one URI per line.
- [ ] Add systemd user-service examples and troubleshooting commands.
- [ ] Update `docs/progress.md` when the phase starts or completes.

Acceptance:

- [ ] A user can start `xrat serve` and verify all v1 endpoints from docs.
- [ ] A user can enable the daemon-hosted API from `config.toml`.
- [ ] A user can run the daemon-hosted API through `systemctl --user`.
- [ ] Security limitations are explicit.
- [ ] Docs match actual response shapes from tests.

### P5.14 Test matrix

Goal: keep coverage focused on behavior and cross-boundary contracts.

Required tests:

- [ ] Config deserialization defaults and overrides.
- [ ] CLI parsing for `serve`.
- [ ] `/health` no-auth response.
- [ ] Auth: no configured key, missing key, wrong key, correct key.
- [ ] `/json` default list, `top`, filter, empty result.
- [ ] `/b64` default output, decoded URI lines, empty result.
- [ ] `/configs` pagination and bounds.
- [ ] `/configs/:id` found, missing, latest-test null.
- [ ] Deleted-state filtering/serialization once soft-delete schema fields exist.
- [ ] Repository top-N ordering.
- [ ] Repository latest-test join chooses the newest test.
- [ ] Server bind failure maps to an app error.
- [ ] Daemon startup honors `server.enabled = false`.
- [ ] Daemon startup attempts HTTP API when `server.enabled = true`.

Manual checks:

- [ ] `cargo build`
- [ ] `cargo test -q`
- [ ] `xrat serve`
- [ ] `curl http://127.0.0.1:8080/health`
- [ ] `curl http://127.0.0.1:8080/json?top=5`
- [ ] `curl http://127.0.0.1:8080/b64`
- [ ] `XRAT_API_KEY=secret xrat daemon start` with `[server].enabled = true`
- [ ] `systemctl --user start xrat-daemon.service`
- [ ] `journalctl --user -u xrat-daemon.service -n 50`

### P5.15 Deferred items

Do not include these in the first Phase 5 delivery unless a later decision
explicitly expands scope:

- [ ] TLS termination.
- [ ] `Authorization: Bearer` auth.
- [ ] CORS configuration.
- [ ] POST/PUT/PATCH/DELETE mutation endpoints.
- [ ] WebSocket or server-sent event streams.
- [ ] HTML dashboard.
- [ ] Prometheus metrics.
- [ ] Request rate limiting.
- [ ] Full test-history endpoints.
- [ ] Backgrounding via `xrat serve --background`.
- [ ] Native systemd installation command such as `xrat service install`.
- [ ] Non-systemd launchd/OpenRC/runit service templates.

## Implementation Progress

Estimated completion: 45%.

Phase 5 has started. The following slices are implemented in code:

- `ServerSettings` exists in `AppConfig` with `enabled`, `host`, `port`, and
  optional `key`.
- `src/server/` exists with Axum router, state, auth, response, error, and route
  modules.
- `GET /health`, `GET /json`, `GET /b64`, `GET /configs`, and
  `GET /configs/{id}` handlers exist.
- optional query-string API key enforcement exists for non-health routes.
- `xrat serve` exists with `--host` and `--port` overrides.
- focused config parsing, CLI parsing, and auth tests exist.

Remaining work before Phase 5 is complete:

- add soft-delete state to config records or explicitly gate deleted-state API
  fields until the Phase 6 config-management slice lands.
- move latest-test joins, top-N real-delay sorting, and pagination into
  repository-level helpers instead of doing route-local in-memory filtering and
  N+1 latest-test queries.
- add route-level tests for `/health`, `/json`, `/b64`, `/configs`, and
  `/configs/{id}`.
- wire `[server].enabled = true` into daemon startup and shutdown.
- extend daemon status output with HTTP API enabled/listening state.
- add systemd user-service examples for daemon-hosted API and standalone
  `xrat serve`.
- verify daemon-hosted serving in an environment that permits sockets, ephemeral
  ports, and runtime process tests.

## Completion Criteria

Phase 5 can be considered complete when:

1. [x] `cargo build` succeeds with new `src/server/` module
2. [ ] `cargo test -q` passes including new server route and DB query tests
3. [x] `xrat serve` starts an HTTP listener on the configured host/port
4. [x] `/health` returns `{"status":"ok"}` without auth
5. [x] `/json` returns a JSON array of configs with test metadata
6. [x] `/json?top=5` returns the 5 fastest configs by real delay
7. [x] `/b64` returns base64-encoded subscription text
8. [x] `/configs` returns paginated config list
9. [x] `/configs/:id` returns a single config with latest test or `404`
10. [x] when `server.key` is set, requests without a matching `?key=` are
        rejected with `401`
11. [x] server shuts down cleanly on SIGINT/SIGTERM
12. [ ] `[server].enabled = true` starts HTTP alongside the daemon
13. [ ] daemon status reports HTTP API enabled/listening state

## Open Questions

These should be resolved while implementing, but they should not block starting
the phase:

- should `/b64` return only `enabled` configs by default? (recommended: yes)
- should `top` require `real_delay_ms` to be present? (recommended: yes)
- should the server support `Authorization: Bearer` header alongside `?key=`?
  (recommended: defer, query param is enough for v1)
- should `xrat serve` support `--background` flag? (recommended: defer, users
  can background with shell tools or use the daemon)
- what is the maximum reasonable `per_page` value? (recommended: 200)
- should the server log requests at all? (recommended: minimal tracing for
  startup/shutdown and errors, no per-request logging in v1)
- should the daemon-hosted server share the same router or run a separate Axum
  instance? (recommended: separate instance, simpler isolation)
- should deletion controls first land in CLI, HTTP, or TUI? (recommended: schema
  and repository first, TUI/CLI controls next, HTTP mutation endpoints deferred)
