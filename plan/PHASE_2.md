# Phase 2 Status

## Scope Reference

`PLAN.md` is not present in the repository root at the moment, so this status is based on `plan/README.md` and the current `src/` implementation.

Phase 2 in `plan/README.md` includes:

- add a SQLite schema for configs and subscription sources
- save parsed configs into the database instead of only writing JSON files
- track metadata such as protocol, address, port, remark/name, source URL, and timestamps
- mark configs as active, disabled, or deleted without physically removing rows

Suggested tables in `plan/README.md`:

- `subscriptions`
- `configs`
- `connection_tests`
- `runtime_sessions`

## Current State

### Implemented

1. **SQLite persistence is now implemented**
   - `Cargo.toml` includes `sqlx` with SQLite support
   - `src/db/` now contains the database layer split into connection, schema, repository, and model modules
   - `src/main.rs` now persists parsed nodes into SQLite instead of only exporting parsed JSON

2. **The domain model supports stable persistence keys**
   - `src/model.rs` defines `Node` with protocol, address, port, credentials, transport fields, and optional display name
   - `src/model.rs` now exposes stable string helpers for `Protocol` and the config dedup key used by the database layer

3. **Import flow already produces normalized nodes before persistence**
   - `src/main.rs` reads input, decodes it, expands URL lists, parses configs, and imports the result into SQLite
   - `src/parser.rs` normalizes fields before deduplication, which is a useful prerequisite for stable database inserts

4. **Subscription source metadata is persisted**
   - `src/io.rs` now classifies input as `url`, `file`, or `raw_text`
   - the database layer stores a subscription row for each import source before inserting related configs

5. **Initial schema and migration history exist**
   - `migrations/0001_init.sql` defines the first SQLite schema for `subscriptions` and `configs`
   - `src/db/schema.rs` uses embedded SQLx migrations so schema updates run automatically when the app starts

6. **Initial persistence behavior is covered by tests**
   - database tests verify subscription creation, config import, upsert behavior, and soft-delete revival
   - parser tests still validate normalization and dedup behavior before persistence

### Not Implemented Yet

1. **JSON config import is not persisted yet**
   - `src/main.rs` currently rejects raw JSON config input for the SQLite import path
   - Phase 2 persistence is focused on parsed subscription-style nodes right now

2. **Only the first two Phase 2 tables are implemented**
   - `subscriptions` and `configs` now exist
   - `connection_tests` and `runtime_sessions` are still only planned models

3. **Read/query APIs are still minimal**
   - the repository currently focuses on import, counting, and a small amount of test support
   - list, filter, selection, and update flows for configs still need to be added

4. **Lifecycle behavior is only partially implemented**
   - soft-delete support exists through `is_deleted` and `deleted_at`
   - broader flows for `is_active`, `is_enabled`, and `is_selected` are not implemented yet

5. **Default database path and app-level setup are still missing**
   - the CLI currently requires the database path to be passed explicitly
   - app data directory behavior has not been designed yet

## Phase 2 Assessment

Phase 2 is **started and meaningfully underway**, with the first usable SQLite persistence path already working.

### Done

- `sqlx` with SQLite support is present in `Cargo.toml`
- parsed nodes already have a structured Rust model
- normalization and deduplication happen before any future persistence step
- `src/db/` exists with a modular database layer
- `migrations/0001_init.sql` defines the initial schema
- parsed nodes are now imported into SQLite
- subscription source records are now stored
- tests cover the first persistence workflows

### Left to finish

- decide whether and how raw JSON configs should be persisted
- add read/query/update flows for configs beyond import
- implement the remaining lifecycle behaviors around config flags
- decide whether `connection_tests` should land in Phase 2 or wait for Phase 3
- decide whether `runtime_sessions` should land in Phase 2 or stay deferred
- decide how `connection_tests` and `runtime_sessions` are introduced during Phase 2 versus later phases

## Proposed Database Models

These models fit the Phase 2 goals in `plan/README.md` while leaving room for Phases 3 and 4.

### `subscriptions`

- `id`: integer primary key
- `source_url`: text, nullable for manually pasted imports
- `source_kind`: enum with values such as `url`, `file`, or `raw_text`
- `name`: text, nullable human-friendly label
- `created_at`: datetime, not null
- `updated_at`: datetime, not null

Purpose:

- represents the origin of imported configs
- allows refresh and audit metadata to be tracked separately from configs

### `configs`

- `id`: integer primary key
- `subscription_id`: integer, nullable foreign key to `subscriptions.id`
- `protocol`: text, not null
- `address`: text, not null
- `port`: integer, not null
- `username`: text, nullable
- `uuid`: text, nullable
- `password`: text, nullable
- `method`: text, nullable
- `network`: text, not null
- `tls`: text, nullable
- `sni`: text, nullable
- `host`: text, nullable
- `path`: text, nullable
- `name`: text, nullable
- `is_active`: boolean, not null, default `false`
- `is_enabled`: boolean, not null, default `true`
- `is_deleted`: boolean, not null, default `false`
- `is_selected`: boolean, not null, default `false`
- `imported_at`: datetime, not null
- `created_at`: datetime, not null
- `updated_at`: datetime, not null
- `deleted_at`: datetime, nullable

Suggested constraints:

- foreign key from `subscription_id` to `subscriptions.id`
- unique index or conflict rule based on the same identity used by `Node::dedup_key()`
- index on `is_active`
- index on `is_enabled`
- index on `(subscription_id, is_deleted)`

Purpose:

- stores normalized proxy configs after parsing
- supports soft deletion and active/disabled state without row removal
- preserves enough transport and credential data to generate runtime configs later

Flag meanings:

- `is_active`: the config is currently being used by the runtime or is considered the live config
- `is_enabled`: the config is allowed to be tested, selected, or exported in normal flows
- `is_deleted`: the config is soft-deleted and should be hidden from normal views without removing the row
- `is_selected`: the config is the current UI or app selection even if it is not actively running

### `connection_tests`

This table is listed in `plan/README.md`, but it is more directly tied to Phase 3. It can be created now or deferred.

- `id`: integer primary key
- `config_id`: integer, not null, foreign key to `configs.id`
- `tcp_ok`: boolean, nullable
- `tcp_ms`: integer, nullable
- `real_delay_ok`: boolean, nullable
- `real_delay_ms`: integer, nullable
- `failure_kind`: enum with values such as `dns`, `timeout`, `refused`, `tls`, `auth`, `process`, or `unknown`
- `failure_reason`: text, nullable
- `tested_at`: datetime, not null

Purpose:

- stores historical connectivity and latency results per config
- keeps machine-friendly failure categories separate from human-readable error details

### `runtime_sessions`

This table is also listed in `plan/README.md`, but it aligns more closely with Phase 4 and is optional unless the app needs runtime persistence.

- `id`: integer primary key
- `config_id`: integer, nullable foreign key to `configs.id`
- `status`: enum with values such as `starting`, `running`, `stopping`, `stopped`, or `failed`
- `mixed_port`: integer, nullable
- `process_id`: integer, nullable
- `started_at`: datetime, nullable
- `stopped_at`: datetime, nullable
- `created_at`: datetime, not null
- `updated_at`: datetime, not null

Purpose:

- only needed if the app must restore or inspect Xray runtime state across restarts
- useful for features like "resume last session", crash recovery hints, or exposing runtime info over the later HTTP API
- if runtime state can stay purely in memory, this table can be postponed or removed from the Phase 2 model set

## Suggested Completion Criteria

Phase 2 can be considered complete after:

1. the project has an initial SQLite schema and reproducible migrations
2. imported subscription sources are stored in a `subscriptions` table
3. parsed nodes are inserted into a `configs` table with stable uniqueness rules
4. config records include core metadata such as protocol, address, port, name, source reference, and timestamps
5. configs can be marked active, disabled, or deleted without physical row removal
6. the current import flow saves to SQLite as the primary persistence path, with JSON export kept only if still intentionally needed

Status against those criteria:

- items 1 through 4 are now partially or substantially implemented
- item 5 is only partially implemented through soft-delete support
- item 6 is implemented for parsed subscription-style imports, but not for raw JSON config import

## Suggested Next Steps

1. add repository methods for listing, selecting, enabling, disabling, and soft-deleting configs
2. decide whether raw JSON configs should be stored directly, converted, or explicitly kept out of the persistence flow
3. add a default database location so users do not need to pass the SQLite path every time
4. decide whether `connection_tests` should be introduced now or deferred cleanly to Phase 3
5. add more tests around config state transitions and query behavior
