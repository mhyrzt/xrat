# Database Backends

xrat supports both SQLite and PostgreSQL as database backends, allowing
flexibility from single-user desktop deployments to multi-user server setups.

## Overview

| Backend        | Use Case                                 | Concurrency        | Setup Complexity   |
| -------------- | ---------------------------------------- | ------------------ | ------------------ |
| **SQLite**     | Single-user, desktop, testing            | Single writer      | Zero configuration |
| **PostgreSQL** | Multi-user, production, high concurrency | Connection pooling | Requires server    |

Both backends use the same schema and support all xrat features.

## Configuration

Configure the database backend in `config.toml`:

```toml
[database]
backend = "sqlite"  # "sqlite" | "postgres"

[database.sqlite]
path = "db.sqlite"

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

## SQLite

SQLite is the default backend, ideal for single-user deployments.

### Advantages

- **Zero configuration**: No server setup required
- **Single file**: Database is a single file on disk
- **Portable**: Easy to backup and move
- **Fast**: Excellent read performance

### Limitations

- **Single writer**: Only one process can write at a time
- **No concurrent access**: Not suitable for multi-user deployments
- **File locking**: "database is locked" errors under high concurrency

### Configuration

```toml
[database]
backend = "sqlite"

[database.sqlite]
path = "db.sqlite"  # relative to config directory or absolute
```

### File Location

The database file is resolved in this order:

1. `--database <path>` CLI flag
2. `[database.sqlite].path` in config.toml
3. `[paths].database` in config.toml (deprecated)
4. `XRAT_PATH/db.sqlite`
5. `~/.config/xrat/db.sqlite`

### Backup

Backup the database file:

```bash
cp ~/.config/xrat/db.sqlite ~/backup/db.sqlite.$(date +%Y%m%d)
```

### Performance Tuning

For better write performance, consider:

- **WAL mode**: Enabled by default in xrat
- **Busy timeout**: Configured internally (5 seconds)
- **Indexing**: Automatic on frequently queried columns

### Troubleshooting

**"database is locked" errors**:

- Only one process can write to SQLite at a time
- Ensure no other xrat processes are running
- Consider PostgreSQL for multi-user deployments

## PostgreSQL

PostgreSQL is recommended for multi-user deployments and high concurrency.

### Advantages

- **Concurrent access**: Multiple readers and writers
- **Connection pooling**: Efficient connection management
- **Scalability**: Handles large datasets and high traffic
- **Reliability**: ACID compliance, crash recovery

### Limitations

- **Server required**: Must install and configure PostgreSQL
- **Network overhead**: Slightly slower than SQLite for single-user
- **Complexity**: More setup and maintenance

### Installation

Install PostgreSQL:

**Ubuntu/Debian**:

```bash
sudo apt install postgresql postgresql-contrib
```

**macOS**:

```bash
brew install postgresql
```

**Docker**:

```bash
docker run -d \
  --name xrat-postgres \
  -e POSTGRES_USER=xrat \
  -e POSTGRES_PASSWORD=secret \
  -e POSTGRES_DB=xrat \
  -p 5432:5432 \
  postgres:15
```

### Setup

1. **Create database and user**:

```bash
sudo -u postgres psql
```

```sql
CREATE USER xrat WITH PASSWORD 'your-password';
CREATE DATABASE xrat OWNER xrat;
GRANT ALL PRIVILEGES ON DATABASE xrat TO xrat;
\q
```

2. **Configure xrat**:

```toml
[database]
backend = "postgres"

[database.postgres]
user = "xrat"
password = "your-password"
host = "localhost"
port = 5432
db_name = "xrat"
max_connections = 10
min_connections = 1
connect_timeout_secs = 10
```

3. **Use environment variables** (recommended):

```toml
[database.postgres]
user = { env = "XRAT_POSTGRES_USER" }
password = { env = "XRAT_POSTGRES_PASSWORD" }
host = "localhost"
port = 5432
db_name = "xrat"
```

```bash
export XRAT_POSTGRES_USER=xrat
export XRAT_POSTGRES_PASSWORD=your-password
xrat import https://example.com/sub.txt
```

### Connection Pooling

xrat uses a connection pool for PostgreSQL:

| Setting                | Description              | Default |
| ---------------------- | ------------------------ | ------- |
| `max_connections`      | Maximum pool size        | `10`    |
| `min_connections`      | Minimum idle connections | `1`     |
| `connect_timeout_secs` | Connection timeout       | `10`    |

Tune based on your workload:

- **Low traffic**: `max_connections = 5`
- **Medium traffic**: `max_connections = 10`
- **High traffic**: `max_connections = 20-50`

### Backup

Use `pg_dump` for backups:

```bash
# Full backup
pg_dump xrat > ~/backup/xrat.$(date +%Y%m%d).sql

# Compressed backup
pg_dump -Fc xrat > ~/backup/xrat.$(date +%Y%m%d).dump

# Restore
pg_restore -d xrat ~/backup/xrat.20260528.dump
```

### Performance Tuning

**PostgreSQL configuration** (`postgresql.conf`):

```ini
# Memory
shared_buffers = 256MB
effective_cache_size = 1GB
work_mem = 16MB

# WAL
wal_level = replica
max_wal_size = 2GB

# Connections
max_connections = 100
```

**Indexing**: xrat automatically creates indexes on:

- `configs.dedup_key` (unique)
- `configs.subscription_id`
- `connection_tests.config_id`
- `connection_tests.run_id`
- `runtime_sessions.config_id`

### High Availability

For production deployments, consider:

- **Replication**: Streaming replication for read replicas
- **Connection pooling**: PgBouncer or Pgpool-II
- **Monitoring**: pg_stat_statements, Prometheus exporter
- **Backups**: Automated daily backups with WAL archiving

## Schema Migrations

xrat uses SQLx for schema migrations. Migrations run automatically on startup:

```rust
sqlx::migrate!("./migrations/sqlite").run(&pool).await?;
```

### Migration Files

Located in `migrations/sqlite/` and `migrations/postgres/`:

```
0001_init.sql
0002_add_connection_test_download_mbps.sql
0003_canonical_config_dedup_key.sql
...
0015_add_config_soft_delete.sql
```

### Manual Migration

If migrations fail, run manually:

```bash
# SQLite
sqlite3 ~/.config/xrat/db.sqlite < migrations/sqlite/0001_init.sql

# PostgreSQL
psql xrat < migrations/postgres/0001_init.sql
```

### Migration Policy

SQLx records the checksum of every applied migration in the `_sqlx_migrations`
table. Editing a migration file changes that checksum, and SQLx would normally
reject the migration history on the next startup. xrat distinguishes two kinds
of edit:

- **Reformatting** (whitespace, line wrapping, `--` comments) is allowed.
  Running `just fmt` over migrations is safe.
- **Changing the meaning** of a migration that has already been applied or
  released is not. Add a new ordered migration instead.

This is enforced at two layers:

- **Runtime**: on startup, xrat records each applied migration's _normalized_
  checksum (comments stripped, whitespace collapsed) in `_xrat_migration_norms`.
  If a migration's stored checksum no longer matches the file but the normalized
  SQL is unchanged, it heals the stored checksum automatically. If the
  normalized SQL changed, startup fails with an actionable error — the meaning
  of an applied migration was altered.
- **CI**: a committed manifest (`migrations/checksums.json`) pins each
  migration's normalized checksum. The test
  `migration_files_match_committed_checksum_manifest` passes through
  reformatting and only fails when a migration changes meaning or a new
  migration is added.

When you add a new migration (or deliberately change one that no database has
applied), regenerate the manifest:

```bash
UPDATE_MIGRATION_MANIFEST=1 cargo test \
  migration_files_match_committed_checksum_manifest
```

### Recovering from a Checksum Mismatch

Reformatting recovers automatically — no action needed. A startup failure means
a migration's _meaning_ changed after it was applied; recovery depends on
whether it shipped.

**Local development database** (the migration is not yet released):

- If the change was intentional, discard the throwaway local database and let
  migrations re-run from scratch, then regenerate the manifest with the command
  above.

**Released user database** (the migration shipped in a published build):

- Do not change the migration's meaning. Restore the original migration file by
  reinstalling the matching release, then add a new migration for any further
  schema change.
- If the database is already broken, restore it from a backup (see the Backup
  sections above).

## Switching Backends

To switch from SQLite to PostgreSQL:

1. **Export data from SQLite**:

```bash
sqlite3 ~/.config/xrat/db.sqlite .dump > xrat-data.sql
```

2. **Convert SQL** (SQLite → PostgreSQL syntax):

```bash
# Manual conversion or use tools like pgloader
pgloader sqlite:///path/to/db.sqlite postgresql://xrat:password@localhost/xrat
```

3. **Update config.toml**:

```toml
[database]
backend = "postgres"
```

4. **Import data**:

```bash
psql xrat < xrat-data-converted.sql
```

## Monitoring

### SQLite

Check database size:

```bash
ls -lh ~/.config/xrat/db.sqlite
```

Check integrity:

```bash
sqlite3 ~/.config/xrat/db.sqlite "PRAGMA integrity_check;"
```

### PostgreSQL

Check connection count:

```sql
SELECT count(*) FROM pg_stat_activity WHERE datname = 'xrat';
```

Check table sizes:

```sql
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

Check slow queries:

```sql
SELECT query, calls, total_time, mean_time
FROM pg_stat_statements
ORDER BY mean_time DESC
LIMIT 10;
```

## Security

### SQLite

- **File permissions**: Restrict access to database file

```bash
chmod 600 ~/.config/xrat/db.sqlite
```

### PostgreSQL

- **Authentication**: Use strong passwords
- **SSL**: Enable SSL for remote connections
- **Firewall**: Restrict access to PostgreSQL port (5432)
- **User permissions**: Use dedicated user with minimal privileges

```sql
-- Read-only user for monitoring
CREATE USER xrat_read WITH PASSWORD 'read-password';
GRANT SELECT ON ALL TABLES IN SCHEMA public TO xrat_read;
```

## Related

- [Deployment](README.md) — deployment overview
- [Database Schema](../05-reference/database-schema.md) — table definitions
- [Configuration](../01-getting-started/configuration.md) — config.toml
  reference
