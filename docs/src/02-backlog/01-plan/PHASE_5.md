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
- `host` / `port`: bind address for the Axum listener.
- `key`: optional `SecretString`. When configured, all requests except `/health`
  must include `?key=<value>` or receive `401`.

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

## Implementation Progress

Estimated completion: 0%.

This phase has not yet started. The following work is planned:

- add `ServerSettings` to `AppConfig` with `enabled`, `host`, `port`, `key`
- create `src/server/` module tree with Axum router and state
- implement `/health`, `/json`, `/b64`, `/configs`, `/configs/:id` endpoints
- add auth middleware for optional API key enforcement
- add DB query helpers for config-with-test joins and pagination
- add `xrat serve` CLI command
- write tests for routes, auth, queries, and CLI parsing

## Completion Criteria

Phase 5 can be considered complete when:

1. [ ] `cargo build` succeeds with new `src/server/` module
2. [ ] `cargo test -q` passes including new server route and DB query tests
3. [ ] `xrat serve` starts an HTTP listener on the configured host/port
4. [ ] `/health` returns `{"status":"ok"}` without auth
5. [ ] `/json` returns a JSON array of configs with test metadata
6. [ ] `/json?top=5` returns the 5 fastest configs by real delay
7. [ ] `/b64` returns base64-encoded subscription text
8. [ ] `/configs` returns paginated config list
9. [ ] `/configs/:id` returns a single config with latest test or `404`
10. [ ] when `server.key` is set, requests without a matching `?key=` are
        rejected with `401`
11. [ ] server shuts down cleanly on SIGINT/SIGTERM

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
