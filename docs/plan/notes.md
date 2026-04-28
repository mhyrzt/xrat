# Code Review Feedback & Refactoring Notes

This file tracks remaining cleanup and improvement work before moving into the
larger phase 4 items. Completed phase 3.6 cleanup work is summarized in
`docs/plan/PHASE_3p6.md`.

## General

### Error Handling

- Avoid excessive use of `Box<dyn std::error::Error>`.
- Prefer concrete error types where possible.
- Consider introducing module-specific error enums using `thiserror`.
- Use `anyhow` only at application boundaries if needed, not throughout
  library/domain code.

### Console Output and Logging

- Tracing is now initialized and internal `eprintln!` diagnostics have been
  migrated.
- Continue to reserve stdout for intentional CLI output only.
- Audit future output changes with these level guidelines:
  - `trace!` for very detailed internal flow
  - `debug!` for debugging information
  - `info!` for normal operational messages
  - `warn!` for recoverable issues
  - `error!` for failures

## Node Deduplication

### `src/model/node_dedup_key`

- Avoid formatting the deduplication key as a string if it is only used for
  identity/deduplication.
- Consider hashing the relevant fields and storing the hash in the database.
- The hash should be deterministic and based only on fields that define node
  uniqueness.

Suggested approach:

- Define a stable struct containing the deduplication fields.
- Serialize/hash the fields in a deterministic order.
- Store the resulting hash as a binary or hex value in the database.

Example fields to consider:

- protocol
- address/host
- port
- user ID / password / credentials
- network type
- TLS/security options
- path/SNI/ALPN/etc. if they affect uniqueness
- or the raw config string, if that becomes the intended identity source

Important constraints:

- Make sure the hash algorithm and input format are stable across versions.
- Consider whether collisions are acceptable. If not, store both the hash and
  the original normalized fields required for verification.

## Verification

### PostgreSQL Real-Backend Testing

- PostgreSQL config shape, pool creation, migrations, and repository dispatch are
  implemented.
- Before treating PostgreSQL support as production-ready, test against a real
  PostgreSQL server.

Suggested configuration:

```toml
[database]
backend = "postgres"

[database.postgres]
user = { env = "XRAT_POSTGRES_USER" }
password = { env = "XRAT_POSTGRES_PASSWORD" }
host = "localhost"
port = 5432
db_name = "xrat"
max_connections = 10
min_connections = 1
connect_timeout_secs = 10
```

Exercise at least:

- schema migration on an empty database
- `import` / `add` upserts
- `list configs` and `list subscriptions`
- selection, activation, enable/disable, and delete state changes
- connection test insertion and latest-history reads
- runtime session insert/update/stop reads

## Tester Improvements

### Parallel Testing

- Current implementation appears to test configurations one by one.
- This will be too slow when testing all configs stored in the database.
- Add parallelization for bulk testing.

Suggested configuration:

```toml
[tester]
concurrency = 16 # i32 or auto, default to auto
```

Implementation options:

- Use `tokio::task`.
- Use `futures::stream::FuturesUnordered`.
- Use `buffer_unordered(concurrency)` for controlled parallelism.

Testing should support bounded concurrency to avoid:

- excessive system resource usage
- too many open sockets
- rate limits
- unstable benchmark results

### Progress Display

- Consider using `indicatif` for progress display during bulk tests.
- Show total configs, completed count, failed count, and average/remaining time
  if useful.
- Progress output should be disabled or adjusted when stdout is intended for
  machine-readable output.

### Test Result Output

- After testing, print results in a structured and pipe-friendly format.
- TSV is a good default because it is human-readable and easy to pipe into
  tools like `sort`, `awk`, and `cut`.

Suggested TSV columns:

```text
id name protocol address port icmp_ms real_delay_ms download_mbps status error
```

Suggested CLI flags:

```text
--format {csv,tsv,json}
--ouptput filename
--sort-by {real-delay, download-speed, icmp}
--no-progress
```

Results should be sortable by:

- status
- ICMP latency
- real delay
- download speed
- protocol
- address

### Connection Testing Flow

- The connection testing pipeline should support the following steps:

```text
ICMP -> Real Delay -> Download Speed
```

- ICMP and download speed tests should be configurable and optionally disabled.

Suggested configuration:

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

## Suggested Refactoring Priorities

1. Introduce stronger error types instead of widespread `Box<dyn Error>`.
2. Add parallel bulk testing with configurable concurrency.
3. Add structured test result output such as TSV/JSON.
4. Improve node deduplication key by using a stable hash-based approach.
5. Verify PostgreSQL support against a real PostgreSQL server.
