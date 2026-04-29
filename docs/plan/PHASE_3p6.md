# Phase 3.6 Cleanup Summary

Phase 3.6 captured the cleanup work completed after phase 3 and before the
larger phase 4 items. The goal was to reduce source-tree clutter, make config
behavior explicit, centralize defaults, and prepare large modules for future
work.

## Completed Changes

### Documentation Layout

- Moved planning and source-adjacent documentation into `docs/`.
- Removed markdown planning files from source directories such as `src/config`.
- Kept source folders focused on Rust code and runtime assets.

Related commit:

```text
3b41b02 feat: add parser config mode
```

### Parser Config Mode

- Added `ParseMode` support to the app config.
- Added `[parser] parse_mode = "strict"` to the example config.
- Clarified that parser mode applies to Xray JSON schema tolerance, not
  subscription/share-link import behavior.

Mode meanings:

- `strict`: reject unknown fields in Xray JSON config parsing.
- `lenient`: accept supported fields and ignore unknown Xray JSON fields.
- `auto`: currently behaves leniently and is reserved for future detection.

Related commit:

```text
3b41b02 feat: add parser config mode
```

### Tester Defaults

- Centralized tester-related defaults in `src/app/config/defaults.rs`.
- Renamed the previous raw-identifier constants module to `defaults` for clearer
  naming.
- Moved real-delay test URL and timeout defaults out of tester modules.
- Moved Xray startup timeout default into the app config defaults module.

Related commit:

```text
f020432 refactor: centralize tester defaults
```

### Tracing Diagnostics

- Added `tracing` and `tracing-subscriber`.
- Initialized tracing in `src/main.rs`.
- Replaced all `eprintln!` diagnostics with tracing macros.
- Kept intentional CLI stdout output as `println!`.
- Default log filtering is `warn`, with `RUST_LOG` override support.
- Added global `--verbose`/`-v` and `--quiet`/`-q` flags for CLI-driven
  diagnostics.
- Documented the ongoing logging level guidelines:
  - `trace!` for very detailed internal flow
  - `debug!` for debugging information
  - `info!` for normal operational messages
  - `warn!` for recoverable issues
  - `error!` for failures

Related commit:

```text
2bcd1d1 refactor: add tracing diagnostics
```

### Tester Bulk Execution and Output

- Refactored `xrat test` so one-config testing returns a structured result
  before printing or persisting.
- Kept `xrat test <id>` working with human-friendly single-config output.
- Made the config ID optional so `xrat test` without an ID runs bulk tests.
- Added bulk filters matching `list configs`:
  - `--enabled-only`
  - `--selected-only`
  - `--active-only`
  - `--subscription <id>`
- Persisted one connection test row for each tested config.
- Added bounded bulk concurrency using Tokio tasks already in the dependency
  tree.
- Added `[testing] concurrency = 0` and `--concurrency <n>`:
  - `0` means auto and is the default.
  - Positive values set exact bounded concurrency.
  - Negative values are invalid.
  - Auto resolves at runtime with a floor of `1` and upper cap of `8`.
- Added structured bulk output:
  - `--format {tsv,json}`
  - `--output <filename>`
  - `--sort-by {status,icmp,real-delay,download-speed,protocol,address}`
  - `--no-progress`
- Added `indicatif` progress bars for bulk tests.
- Kept stdout pipe-friendly by writing bulk progress to stderr.
- Added stage enablement config:
  - `[testing.icmp].enabled`
  - `[testing.icmp].attempts`
  - `[testing.real_delay].enabled`
  - `[testing.download].enabled`
  - `[testing.tcp].enabled`
- Updated `docs/plan/config.example.toml` with the new tester config shape.
- Left actual download-speed checks and download Mbps persistence for follow-up.

### Configurable Connection Test Flow

- Added `[testing].order` to configure the user-facing connection test stage
  order.
- Added `[testing].failure_policy` to control what happens after a failed stage:
  - `continue` keeps testing later configured stages.
  - `skip_remaining` stops testing after the failure.
  - `mark_failed` stops testing and records the node as failed.
- Defaulted the ordered flow to:

```text
ICMP -> Real Delay -> Download Speed
```

- Kept TCP as an internal gate that runs immediately before real-delay when TCP
  checks are enabled.
- Left TCP persistence in place for now so existing connection-test history and
  repository behavior stay compatible.

### Database Module Facade

- Split the `Database` facade out of `src/db/mod.rs` into `src/db/database.rs`.
- Reduced `src/db/mod.rs` to module declarations and public re-exports.
- Moved database facade tests with the facade implementation.

Related commit:

```text
8762c10 refactor: split database facade module
```

### Repository Module Facade

- Split public repository facade functions out of `src/db/repository/mod.rs`
  into `src/db/repository/facade.rs`.
- Kept domain repository files separated by entity:
  - `configs.rs`
  - `connection_tests.rs`
  - `runtime_sessions.rs`
  - `subscriptions.rs`
- Reduced `src/db/repository/mod.rs` to module declarations and public
  re-exports.

Related commit:

```text
bf30bc5 refactor: split repository facade module
```

### Xray Runtime Config Module

- Split `src/xray/config.rs` into a folder module.
- Moved serializable config structs into `src/xray/config/types.rs`.
- Moved outbound settings generation into `src/xray/config/outbound.rs`.
- Moved stream settings generation into `src/xray/config/stream.rs`.
- Kept the public API available through `src/xray/config/mod.rs`.

### Xray Shared Config Types

- Split `src/config/xray/shared.rs` into a folder module.
- Moved shared type aliases into `src/config/xray/shared/aliases.rs`.
- Moved network/security enums into `src/config/xray/shared/network.rs`.
- Moved log-related enums into `src/config/xray/shared/logging.rs`.
- Moved DNS/domain strategy enums into `src/config/xray/shared/strategy.rs`.
- Moved untagged range value enums into `src/config/xray/shared/ranges.rs`.
- Kept existing imports available through `src/config/xray/shared/mod.rs`.

### Database Backend Support

- Added a first PostgreSQL backend slice alongside SQLite.
- Added `[database]`, `[database.sqlite]`, and `[database.postgres]` app config
  sections.
- Kept the legacy `[paths].database` SQLite path as a compatibility alias.
- Configured PostgreSQL using separate `user`, `password`, `host`, `port`, and
  `db_name` fields instead of a raw URL.
- Allowed PostgreSQL `user` and `password` to use the existing secret/env value
  format.
- Enabled SQLx's PostgreSQL feature and added a `DatabaseConnectionConfig` plus
  backend-dispatched pool enum.
- Added PostgreSQL migrations under `migrations/postgres/`.
- Updated repository functions to dispatch across SQLite and PostgreSQL pools.
- Added shared row-mapping helpers for backend-neutral record mapping.
- Redacted PostgreSQL passwords in user-facing database labels.

Carry-forward note: PostgreSQL compiles and the SQLite suite still passes, but
this still needs verification against a real PostgreSQL server.

### Concrete Error Types

- Added `src/db/error.rs` with a concrete `DbError` enum and DB-local
  `Result<T>` alias.
- Replaced DB-layer `Box<dyn std::error::Error>` return types with
  `crate::db::Result<T>`.
- Preserved SQLx, migration, and filesystem error propagation through
  `thiserror` variants.
- Converted invalid runtime-session status row mapping into a typed
  `DbError::InvalidRuntimeSessionStatus` variant.
- Added `src/app/error.rs` with a concrete `AppError` enum and app-local
  `Result<T>` alias.
- Replaced app-layer and `src/main.rs` boxed error return types with
  `crate::app::Result<T>` / `xrat::app::Result<T>`.
- Converted stringly application failures into typed variants for import
  validation, path resolution, PostgreSQL config validation, and unsupported
  protocol reconstruction.

Related commit:

```text
9f27cc5 refactor: add concrete db error type
```

App-layer error refactor is complete locally and should be committed with this
documentation update.

## Validation

Each implementation commit was validated with:

```text
cargo fmt
cargo test -q
```

The last validation run reported 83 passing tests.

## Carry Forward

The remaining notes are kept in `docs/plan/notes.md`. Near-term carry-forward
items are real PostgreSQL backend verification, node deduplication hashing, and
tester workflow improvements.
