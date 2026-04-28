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
- Renamed the previous raw-identifier constants module to `defaults` for
  clearer naming.
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

Related commit:

```text
2bcd1d1 refactor: add tracing diagnostics
```

### Database Module Facade

- Split the `Database` facade out of `src/db/mod.rs` into
  `src/db/database.rs`.
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

## Validation

Each implementation commit was validated with:

```text
cargo fmt
cargo test -q
```

The last validation run reported 78 passing tests.

## Carry Forward

The remaining notes are kept in `docs/plan/notes.md`. Near-term carry-forward
items are real PostgreSQL backend verification, stronger error types, node
deduplication hashing, and tester workflow improvements.
