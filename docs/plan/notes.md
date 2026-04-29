# Code Review Feedback & Refactoring Notes

This file tracks remaining cleanup and improvement work before moving into the
larger phase 4 items. Completed phase 3.6 cleanup work is summarized in
`docs/plan/PHASE_3p6.md`.

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

- PostgreSQL config shape, pool creation, migrations, and repository dispatch
  are implemented.
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

## Suggested Refactoring Priorities

1. Improve node deduplication key by using a stable hash-based approach.
2. Verify PostgreSQL support against a real PostgreSQL server.
