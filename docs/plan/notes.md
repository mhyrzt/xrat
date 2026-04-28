# Code Review Feedback & Refactoring Notes

## General

- Avoid excessive use of `Box<dyn std::error::Error>`.
  - Prefer concrete error types where possible.
  - Consider introducing module-specific error enums using `thiserror`.
  - Use `anyhow` only at application boundaries if needed, not throughout library/domain code.

- Replace `println!` / `eprintln!` with proper logging.
  - [x] Add and initialize the `tracing` crate for structured logging.
  - [x] Replace internal parser/process diagnostics with tracing.
  - Audit the codebase and replace direct console output where appropriate.
  - Reserve stdout for intentional CLI output only.
  - Use log levels consistently:
    - `trace!` for very detailed internal flow
    - `debug!` for debugging information
    - `info!` for normal operational messages
    - `warn!` for recoverable issues
    - `error!` for failures

## `src/xray/config.rs`

- This file is too large and should be split into smaller modules.
- Remove unnecessary comments.
- The code should be self-explanatory through clear naming and smaller functions/types.
- Keep comments only where they explain non-obvious behavior, external constraints, or protocol-specific details.

## `src/model/node_dedup_key`

- Avoid formatting the deduplication key as a string if it is only used for identity/deduplication.

- Consider hashing the relevant fields instead and storing the hash in the database.
  - This may be cleaner, more compact, and better suited for indexing.
  - The hash should be deterministic and based only on fields that define node uniqueness.

- Suggested approach:
  - Define a stable struct containing the deduplication fields.
  - Serialize/hash the fields in a deterministic order.
  - Store the resulting hash as a binary or hex value in the database.

- Example fields to consider:
  - protocol
  - address/host
  - port
  - user ID / password / credentials
  - network type
  - TLS/security options
  - path/SNI/ALPN/etc. if they affect uniqueness

- Important:
  - Make sure the hash algorithm and input format are stable across versions.
  - Consider whether collisions are acceptable. If not, store both the hash and the original normalized fields required for verification.

## `src/db`

- Add support for PostgreSQL connections.

- This will require changes in:
  - database abstraction layer
  - migrations
  - configuration
  - dependencies
  - repository implementations if they are currently SQLite-specific

- `config.toml` should support selecting the database backend.

- Suggested configuration:

  ```toml
  [database]
  backend = "sqlite" # sqlite | postgres

  [database.sqlite]
  path = "data/app.db" or whatever is default

  [database.postgres]
  url = "postgres://user:password@localhost:5432/database"
  max_connections = 10
  min_connections = 1
  connect_timeout_secs = 10
  ```

- Suggested dependency changes:
  - If using `sqlx`, enable both SQLite and PostgreSQL features.
  - Make sure migrations are compatible with both databases or are separated by backend.

## `src/db/mod.rs`

- `src/db/mod.rs` is too large for a module entry file.

- Keep `mod.rs` focused on:
  - module declarations
  - public re-exports
  - high-level initialization functions only

## `src/db/repository/mod.rs`

- `src/db/repository/mod.rs` is too large.

- Split repository code into separate files by domain/entity.

- Suggested structure:

  ```text
  src/db/repository/
    mod.rs
    node.rs
    node_test.rs
    subscription.rs
    profile.rs
    common.rs
  ```

- `mod.rs` should only contain module declarations and re-exports.

## `src/config`

- [x] There were too many `.md` files inside `src/config`.
- [x] Moved documentation files to a root-level `docs/` directory.
- [x] Source directories now mainly contain source code, not documentation.

## `config.toml`

- [x] Add a configuration field for `ParseMode`.
- [x] `ParseMode` controls Xray JSON schema tolerance, not subscription import behavior.

## `src/config/shared.rs`

- `shared.rs` is too large and should be converted into a folder with smaller files.

## `src/tester`

- [x] Move hardcoded constants from tester modules into application configuration constants.

- Specifically:
  - [x] In the real delay tester file, move the existing constant to:

    ```text
    src/app/config/defaults.rs
    ```

- [x] Also reviewed the tester module for other hardcoded values that should be centralized.

- Examples of values that may belong in `app/config/defaults.rs`:
  - [x] default timeout durations
  - [x] default test URLs
  - default retry counts
  - default buffer sizes
  - default concurrency limits
  - default ICMP settings
  - default real-delay settings
  - default download-speed settings

## `src/app/commands/test.rs`

### Parallel Testing

- Current implementation appears to test configurations one by one.
- This will be too slow when testing all configs stored in the database.
- Add parallelization for bulk testing.
- Suggested configuration:

  ```toml
  [tester]
  concurrency = 16 # i32 or auto, default to auto
  ```

- Implementation options:
  - Use `tokio::task`
  - Use `futures::stream::FuturesUnordered`
  - Use `buffer_unordered(concurrency)` for controlled parallelism

- Testing should support bounded concurrency to avoid:
  - excessive system resource usage
  - too many open sockets
  - rate limits
  - unstable benchmark results

### Progress Display

- Consider using `indicatif` for progress display during bulk tests.

- This would improve UX for long-running test operations.

- Example behavior:
  - show total number of configs
  - show completed count
  - show failed count
  - show average/remaining time if possible

- Progress output should be disabled or adjusted when stdout is intended for machine-readable output.

---

### Test Result Output

- After testing, print results in a structured and pipe-friendly format.

- TSV is a good default because it is:
  - human-readable
  - easy to parse
  - easy to pipe into tools like `sort`, `awk`, `cut`, etc.

- Suggested TSV columns:

  ```text
  id name protocol address port icmp_ms real_delay_ms download_mbps status error
  ```

- Example output:

  ```text
  12 node-a vmess example.com 443 45 210 18.4 ok 
  17 node-b trojan example.org 443  0 0 failed timeout
  ```

- Results should be sortable by:
  - status
  - ICMP latency
  - real delay
  - download speed
  - protocol
  - address

- Consider adding CLI flags:

  ```text
  --output tsv
  --output json
  --sort-by real-delay
  --sort-by download-speed
  --no-progress
  ```

---

## Connection Testing Flow

- The connection testing pipeline should support the following steps:

  ```text
  ICMP -> Real Delay -> Download Speed
  ```

- ICMP and download speed tests should be configurable and optionally disabled.

- Suggested configuration:

  ```toml
  [tester.icmp]
  enabled = true
  timeout_ms = 1000
  attempts = 3

  [tester.real_delay]
  enabled = true
  timeout_ms = 5000
  test_url = "https://www.gstatic.com/generate_204"

  [tester.download_speed]
  enabled = false
  timeout_ms = 10000
  test_url = "https://speed.cloudflare.com/__down?bytes=10485760"
  ```

- The test runner should skip disabled stages cleanly.

- If a required earlier stage fails, behavior should be configurable:
  - continue testing remaining stages
  - skip remaining stages
  - mark node as failed

---

## Suggested Refactoring Priorities

1. Replace `println!`/`eprintln!` with `tracing`.
2. Split large modules:
   - `src/xray/config.rs`
   - `src/db/mod.rs`
   - `src/db/repository/mod.rs`
   - `src/config/shared.rs`
3. Introduce stronger error types instead of widespread `Box<dyn Error>`.
4. [done] Move documentation files from `src/config` to root-level `docs/`.
5. [done] Add `ParseMode` support to `config.toml`.
6. [done] Move tester constants into `src/app/config/defaults.rs`.
7. Add PostgreSQL support.
8. Improve node deduplication key by using a stable hash-based approach.
9. Add parallel bulk testing with configurable concurrency.
10. Add structured test result output such as TSV/JSON.
