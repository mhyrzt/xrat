# Database Schema

xrat uses relational databases (SQLite or PostgreSQL) with the same schema
across both backends.

## Schema Overview

| Table                  | Version                                  | Description                   |
| ---------------------- | ---------------------------------------- | ----------------------------- |
| `subscriptions`        | 0001                                     | Import source tracking        |
| `configs`              | 0001, 0003, 0015                         | Stored proxy nodes            |
| `connection_tests`     | 0001, 0002, 0008, 0009, 0010             | Test results per config       |
| `connection_test_runs` | 0007                                     | Groups test results into runs |
| `runtime_sessions`     | 0001, 0004, 0005, 0006, 0012, 0013, 0014 | Proxy process lifecycle       |
| `cf_scan_results`      | 0011                                     | IP scan results               |
| `events`               | 0017                                     | App/runtime event log         |

---

## Tables

### subscriptions

Tracks import sources (URLs, files, raw text).

```sql
CREATE TABLE subscriptions (
    id INTEGER PRIMARY KEY,
    ref TEXT UNIQUE,
    source_url TEXT,
    source_kind TEXT NOT NULL CHECK(source_kind IN ('url', 'file', 'raw_text')),
    name TEXT,
    last_refreshed_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

| Column              | Type      | Description                            |
| ------------------- | --------- | -------------------------------------- |
| `id`                | INTEGER   | Primary key                            |
| `ref`               | TEXT      | Stable user-facing ref                 |
| `source_url`        | TEXT      | Original URL, file path, or "raw_text" |
| `source_kind`       | TEXT      | `url`, `file`, or `raw_text`           |
| `name`              | TEXT      | Optional subscription name             |
| `last_refreshed_at` | TIMESTAMP | Latest successful refresh timestamp    |
| `created_at`        | TIMESTAMP | First import timestamp                 |
| `updated_at`        | TIMESTAMP | Latest import timestamp                |

---

### configs

Stores normalized proxy nodes.

```sql
CREATE TABLE configs (
    id INTEGER PRIMARY KEY,
    ref TEXT UNIQUE,
    subscription_id INTEGER REFERENCES subscriptions(id),
    dedup_key TEXT NOT NULL UNIQUE,
    protocol TEXT NOT NULL,
    address TEXT NOT NULL,
    port INTEGER NOT NULL,
    username TEXT,
    uuid TEXT,
    password TEXT,
    method TEXT,
    network TEXT NOT NULL,
    tls TEXT,
    sni TEXT,
    host TEXT,
    path TEXT,
    name TEXT,
    raw_config TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 0,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    imported_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

| Column            | Type      | Description                                               |
| ----------------- | --------- | --------------------------------------------------------- |
| `id`              | INTEGER   | Primary key                                               |
| `ref`             | TEXT      | Stable user-facing ref                                    |
| `subscription_id` | INTEGER   | FK to subscriptions                                       |
| `dedup_key`       | TEXT      | Unique deduplication key                                  |
| `protocol`        | TEXT      | `vless`, `vmess`, `ss`, `trojan`, `http`, `socks5`, `hy2` |
| `address`         | TEXT      | Server address                                            |
| `port`            | INTEGER   | Server port                                               |
| `username`        | TEXT      | Username (HTTP/SOCKS5)                                    |
| `uuid`            | TEXT      | UUID (VLESS/VMess)                                        |
| `password`        | TEXT      | Password (Trojan/SS)                                      |
| `method`          | TEXT      | Encryption method (Shadowsocks)                           |
| `network`         | TEXT      | `tcp`, `ws`, `grpc`, `udp`                                |
| `tls`             | TEXT      | `tls` or NULL                                             |
| `sni`             | TEXT      | SNI hostname                                              |
| `host`            | TEXT      | Host header (WebSocket)                                   |
| `path`            | TEXT      | Path (WebSocket/gRPC/TCP)                                 |
| `name`            | TEXT      | Display name                                              |
| `raw_config`      | TEXT      | Original raw config line                                  |
| `is_active`       | BOOLEAN   | Currently active runtime config                           |
| `is_enabled`      | BOOLEAN   | Included in bulk operations                               |
| `imported_at`     | TIMESTAMP | Import timestamp                                          |
| `is_deleted`      | BOOLEAN   | Soft-deleted flag                                         |
| `deleted_at`      | TIMESTAMP | Deletion timestamp                                        |
| `created_at`      | TIMESTAMP | Insertion timestamp                                       |
| `updated_at`      | TIMESTAMP | Last update timestamp                                     |

**Indexes**:

- `dedup_key` — UNIQUE
- `subscription_id` — FK index
- `is_enabled`, `is_active` — filter queries
- `is_deleted` — soft-delete queries

---

### connection_tests

Stores individual test results per config.

```sql
CREATE TABLE connection_tests (
    id INTEGER PRIMARY KEY,
    run_id INTEGER REFERENCES connection_test_runs(id),
    config_id INTEGER NOT NULL REFERENCES configs(id),
    icmp_ok INTEGER,
    icmp_ms INTEGER,
    tcp_ok INTEGER,
    tcp_ms INTEGER,
    real_delay_ok INTEGER,
    real_delay_ms INTEGER,
    connect_ms INTEGER,
    ttfb_ms INTEGER,
    http_status INTEGER,
    download_mbps REAL,
    upload_mbps REAL,
    failure_kind TEXT,
    failure_reason TEXT,
    dial_endpoint_ip TEXT,
    dial_endpoint_location TEXT,
    dial_endpoint_country TEXT,
    dial_endpoint_asn TEXT,
    dial_endpoint_geoip_source TEXT,
    dial_endpoint_fronting TEXT,
    tested_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

| Column              | Type      | Description                |
| ------------------- | --------- | -------------------------- |
| `id`                | INTEGER   | Primary key                |
| `run_id`            | INTEGER   | FK to connection_test_runs |
| `config_id`         | INTEGER   | FK to configs              |
| `icmp_ok`           | BOOLEAN   | ICMP ping success          |
| `icmp_ms`           | INTEGER   | ICMP latency               |
| `tcp_ok`            | BOOLEAN   | TCP connect success        |
| `tcp_ms`            | INTEGER   | TCP latency                |
| `real_delay_ok`     | BOOLEAN   | HTTP round-trip success    |
| `real_delay_ms`     | INTEGER   | HTTP round-trip latency    |
| `connect_ms`        | INTEGER   | TCP connect time           |
| `ttfb_ms`           | INTEGER   | Time to first byte         |
| `http_status`       | INTEGER   | HTTP response status       |
| `download_mbps`     | REAL      | Download throughput        |
| `upload_mbps`       | REAL      | Upload throughput          |
| `failure_kind`      | TEXT      | Failure classification     |
| `failure_reason`    | TEXT      | Human-readable error       |
| `dial_endpoint_ip`            | TEXT      | Resolved dial-endpoint IP        |
| `dial_endpoint_location`      | TEXT      | Dial-endpoint GeoIP location     |
| `dial_endpoint_country`       | TEXT      | Dial-endpoint country ISO code   |
| `dial_endpoint_asn`           | TEXT      | Dial-endpoint ASN identifier     |
| `dial_endpoint_geoip_source`  | TEXT      | Lookup provenance (`literal_ip`/`dial_dns`) |
| `dial_endpoint_fronting`      | TEXT      | Detected CDN/relay provider (hint) |
| `tested_at`         | TIMESTAMP | Test timestamp             |

**Indexes**:

- `config_id` — per-config queries
- `run_id` — per-run queries
- `(config_id, tested_at)` — latest test per config

---

### connection_test_runs

Groups test results into batches.

```sql
CREATE TABLE connection_test_runs (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

| Column       | Type      | Description                            |
| ------------ | --------- | -------------------------------------- |
| `id`         | INTEGER   | Primary key                            |
| `kind`       | TEXT      | Run description (e.g., "bulk", "ping") |
| `created_at` | TIMESTAMP | Run timestamp                          |

---

### runtime_sessions

Tracks proxy process lifecycle.

```sql
CREATE TABLE runtime_sessions (
    id INTEGER PRIMARY KEY,
    config_id INTEGER REFERENCES configs(id),
    status TEXT NOT NULL
        CHECK(status IN ('starting', 'running', 'stopping', 'stopped', 'failed')),
    socks_host TEXT,
    socks_port INTEGER,
    http_host TEXT,
    http_port INTEGER,
    shadowsocks_host TEXT,
    shadowsocks_port INTEGER,
    process_id INTEGER,
    failure_reason TEXT,
    owner_kind TEXT,
    owner_instance_id TEXT,
    last_transition_reason_code TEXT,
    last_transition_reason_detail TEXT,
    last_transition_origin TEXT,
    cooldown_until TEXT,
    last_failed_at TEXT,
    last_failed_reason_code TEXT,
    started_at TIMESTAMP,
    stopped_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

| Column                          | Type      | Description                                                |
| ------------------------------- | --------- | ---------------------------------------------------------- |
| `id`                            | INTEGER   | Primary key                                                |
| `config_id`                     | INTEGER   | FK to configs                                              |
| `status`                        | TEXT      | `starting`, `running`, `stopping`, `stopped`, `failed`     |
| `socks_host`                    | TEXT      | SOCKS inbound host                                         |
| `socks_port`                    | INTEGER   | SOCKS inbound port                                         |
| `http_host`                     | TEXT      | HTTP inbound host (if enabled)                             |
| `http_port`                     | INTEGER   | HTTP inbound port                                          |
| `shadowsocks_host`              | TEXT      | Shadowsocks inbound host (if enabled)                      |
| `shadowsocks_port`              | INTEGER   | Shadowsocks inbound port                                   |
| `process_id`                    | INTEGER   | OS process ID                                              |
| `failure_reason`                | TEXT      | Error message (if failed)                                  |
| `owner_kind`                    | TEXT      | `cli` or `daemon`                                          |
| `owner_instance_id`             | TEXT      | Daemon instance UUID                                       |
| `last_transition_reason_code`   | TEXT      | Machine-readable transition reason code                    |
| `last_transition_reason_detail` | TEXT      | Human-readable transition details                          |
| `last_transition_origin`        | TEXT      | Transition source such as CLI, daemon, health, or rotation |
| `cooldown_until`                | TEXT      | Rotation cooldown expiry as epoch seconds                  |
| `last_failed_at`                | TEXT      | Last runtime/health failure time as epoch seconds          |
| `last_failed_reason_code`       | TEXT      | Machine-readable last failure reason code                  |
| `started_at`                    | TIMESTAMP | Session start timestamp                                    |
| `stopped_at`                    | TIMESTAMP | Session stop timestamp                                     |
| `created_at`                    | TIMESTAMP | Record creation timestamp                                  |
| `updated_at`                    | TIMESTAMP | Last update timestamp                                      |

**Indexes**:

- `config_id` — per-config queries
- `status` — running session lookup
- `(owner_kind, owner_instance_id)` — daemon reattach queries

---

### cf_scan_results

Stores IP scan results.

```sql
CREATE TABLE cf_scan_results (
    id INTEGER PRIMARY KEY,
    ip TEXT NOT NULL UNIQUE,
    latency_ms INTEGER,
    download_mbps REAL,
    upload_mbps REAL,
    error TEXT,
    last_scanned_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

| Column            | Type      | Description                       |
| ----------------- | --------- | --------------------------------- |
| `id`              | INTEGER   | Primary key                       |
| `ip`              | TEXT      | IP address (unique)               |
| `latency_ms`      | INTEGER   | Connection latency                |
| `download_mbps`   | REAL      | Download throughput (if measured) |
| `upload_mbps`     | REAL      | Upload throughput (if measured)   |
| `error`           | TEXT      | Error message (if failed)         |
| `last_scanned_at` | TIMESTAMP | Last scan timestamp               |

---

### events

Structured application/runtime event log surfaced by `xrat logs`.

```sql
CREATE TABLE events (
    id INTEGER PRIMARY KEY,
    level TEXT NOT NULL,
    source TEXT NOT NULL,
    kind TEXT NOT NULL,
    config_id INTEGER,
    session_id INTEGER,
    message TEXT NOT NULL,
    detail TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

| Column       | Type    | Description                                              |
| ------------ | ------- | -------------------------------------------------------- |
| `id`         | INTEGER | Primary key                                              |
| `level`      | TEXT    | `info`, `warn`, or `error`                               |
| `source`     | TEXT    | `daemon`, `runtime`, `rotation`, `health`, or `test`     |
| `kind`       | TEXT    | Event kind (e.g. `proxy_rotated`, `connect`, `test_run`) |
| `config_id`  | INTEGER | Related config, if any                                   |
| `session_id` | INTEGER | Related runtime session, if any                          |
| `message`    | TEXT    | Human-readable summary                                   |
| `detail`     | TEXT    | Optional JSON detail                                     |
| `created_at` | TEXT    | Creation timestamp                                       |

Rows are inserted fire-and-forget; a failed insert is logged but never breaks
the operation that produced the event.

---

## Migrations

Migrations are run automatically on startup using SQLx.

### Migration List

| #    | File                                              | Description                                                                |
| ---- | ------------------------------------------------- | -------------------------------------------------------------------------- |
| 0001 | `init.sql`                                        | Initial schema: subscriptions, configs, connection_tests, runtime_sessions |
| 0002 | `add_connection_test_download_mbps.sql`           | Add `download_mbps` to connection_tests                                    |
| 0003 | `canonical_config_dedup_key.sql`                  | Add `dedup_key` to configs                                                 |
| 0004 | `add_runtime_session_inbound_ports.sql`           | Add inbound port columns to runtime_sessions                               |
| 0005 | `drop_runtime_session_mixed_port.sql`             | Clean up mixed port column                                                 |
| 0006 | `add_runtime_session_failure_reason.sql`          | Add failure tracking to runtime_sessions                                   |
| 0007 | `add_connection_test_runs.sql`                    | Add connection_test_runs table                                             |
| 0008 | `add_connection_test_http_fields.sql`             | Add HTTP fields (connect_ms, ttfb_ms, http_status)                         |
| 0009 | `add_connection_test_country_asn.sql`             | Add GeoIP fields (country, ASN)                                            |
| 0010 | `add_connection_test_upload_mbps.sql`             | Add upload_mbps to connection_tests                                        |
| 0011 | `add_cf_scan_results.sql`                         | Add cf_scan_results table                                                  |
| 0012 | `add_runtime_session_owner_transition_fields.sql` | Add owner tracking to runtime_sessions                                     |
| 0013 | `add_runtime_session_transition_origin.sql`       | Add transition origin tracking                                             |
| 0014 | `add_runtime_session_cooldown_failure_fields.sql` | Add cooldown and failure tracking                                          |
| 0015 | `add_config_soft_delete.sql`                      | Add soft-delete fields to configs                                          |
| 0016 | `drop_config_is_selected.sql`                     | Drop the legacy `is_selected` column from configs                          |
| 0017 | `add_events.sql`                                  | Add events table for the `xrat logs` event log                             |
| 0018 | `add_subscription_last_refreshed_at.sql`          | Add subscription refresh timestamp tracking                                |
| 0019 | `add_config_subscription_refs.sql`                | Add stable user-facing refs to configs and subscriptions                   |

### Migration Location

```
migrations/sqlite/0001_init.sql
migrations/sqlite/0002_add_connection_test_download_mbps.sql
...
migrations/sqlite/0015_add_config_soft_delete.sql
```

PostgreSQL migrations have equivalent files in `migrations/postgres/`.

## Schema Diagram

```
subscriptions
    │
    ├── configs (1:N via subscription_id)
    │       │
    │       ├── connection_tests (1:N via config_id)
    │       │       │
    │       │       └── connection_test_runs (1:N via run_id)
    │       │
    │       └── runtime_sessions (1:N via config_id)
    │
    └── (none)

cf_scan_results (standalone, not linked to configs)
```
