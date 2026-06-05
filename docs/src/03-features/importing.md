# Importing

xrat imports proxy configurations from multiple sources and formats,
automatically detecting the input type and normalizing all configs into a
unified internal representation.

## Input Sources

### Subscription URL

Fetch configs from a remote HTTP endpoint:

```bash
xrat import https://example.com/subscription
```

xrat:

1. Fetches the URL content
2. Parses `subscription-userinfo` headers for metadata (upload, download, total,
   expire)
3. Detects format (base64, plain list, JSON)
4. Parses and normalizes each node
5. Persists to database with subscription tracking

### Local File

Import from a file on disk:

```bash
xrat import ./nodes.txt
```

Supports the same format detection as URLs.

### Raw Text

Import inline subscription text:

```bash
xrat import "vless://uuid@example.com:443?type=ws#Node"
```

Useful for quick imports or scripting.

## Input Formats

### Single Share Link

A single proxy URI:

```
vless://uuid-123@example.com:443?type=ws&security=tls&sni=cdn.example.com&path=%2Fray#My%20Node
```

Supported schemes:

- `vless://`
- `vmess://`
- `ss://`
- `trojan://`
- `http://` / `https://`
- `socks5://`
- `hysteria2://` / `hy2://`

### Base64 Subscription

Standard v2rayN/Clash subscription format:

```
dmxlc3M6Ly91dWlkQGV4YW1wbGUuY29tOjQ0Mz90eXBlPXdzJnNlY3VyaXR5PXRscyNNeSBOb2RlCnZtZXNzOi8v...
```

xrat:

1. Base64-decodes the payload
2. Splits into lines
3. Parses each line as a share link

### Plain Link List

Multiple share links, one per line:

```
vless://uuid-1@example.com:443?type=tcp#Node1
vmess://eyJhZGQiOiJleGFtcGxlLmNvbSIsInBvcnQiOiI0NDMifQ==#Node2
ss://YWVzLTI1Ni1nY206c2VjcmV0@example.com:8388#Node3
```

Lines starting with `#` are treated as comments and skipped.

### SIP008 JSON

Shadowsocks SIP008 format:

```json
{
  "version": 1,
  "servers": [
    {
      "server": "example.com",
      "server_port": 8388,
      "method": "aes-256-gcm",
      "password": "secret",
      "remarks": "My SS Node"
    }
  ]
}
```

### Xray JSON

Full Xray configuration:

```json
{
  "inbounds": [...],
  "outbounds": [
    {
      "protocol": "vless",
      "settings": {
        "vnext": [...]
      }
    }
  ]
}
```

xrat extracts outbound configs and converts them to internal nodes.

## Format Detection

xrat automatically detects the input format using heuristics:

| Condition                                                | Detected Format     |
| -------------------------------------------------------- | ------------------- |
| Starts with `{` and contains `"version"` or `"inbounds"` | Xray JSON           |
| Starts with `{` and contains `"servers"`                 | SIP008 JSON         |
| Single line starting with a protocol scheme              | Single share link   |
| Multiple lines, first line starts with protocol scheme   | Plain link list     |
| Otherwise                                                | Base64 subscription |

## Normalization

After parsing, xrat normalizes each node:

1. **Network defaults**: Empty network → `tcp`
2. **WebSocket defaults**: Missing `host` → copy from `sni`, missing `path` →
   `/`
3. **gRPC defaults**: Missing `path` → `/`
4. **TLS cleanup**: Empty string `tls` → `None`

## Deduplication

Before persisting, xrat generates a dedup key for each node and skips
duplicates. See [Deduplication](deduplication.md) for details.

## Subscription Tracking

Each import creates or updates a `subscriptions` record:

| Field               | Description                               |
| ------------------- | ----------------------------------------- |
| `source_url`        | Original URL or file path                 |
| `source_kind`       | `url`, `file`, or `raw_text`              |
| `name`              | Optional name (from URL or user-provided) |
| `created_at`        | First import timestamp                    |
| `updated_at`        | Latest import timestamp                   |
| `last_refreshed_at` | Last successful URL refresh (epoch secs)  |

Configs are linked to their subscription via `subscription_id` foreign key.

## Refreshing Subscriptions

Re-importing a URL-backed subscription is a reconciliation, not an additive
import: configs the provider still returns are upserted, and configs that
disappeared from the payload are soft-deleted (recoverable; a later re-add
restores them). An empty payload removes nothing. See
[Deduplication](deduplication.md) for the dedup key used to match configs.

There are two ways to refresh:

- **Manual** — re-run `xrat import <url>`, or press `r` / `R` on the TUI Sources
  tab. Available any time, no daemon required.
- **Automatic** — the daemon periodically re-fetches URL-backed subscriptions on
  a fixed interval. Configure it under `[subscriptions]`:

  ```toml
  [subscriptions]
  auto_refresh = false
  refresh_interval_hours = 24
  ```

  When `auto_refresh` is enabled, the daemon refreshes each URL-backed
  subscription whose `last_refreshed_at` is older than `refresh_interval_hours`
  (or that was never refreshed). Because the due check reads the persisted
  `last_refreshed_at`, intervals survive daemon restarts. Non-URL sources
  (files, raw text) are skipped, and a failed fetch is recorded as an event
  without stopping the daemon or the rest of the batch. Refresh start, success,
  and failure are visible in `xrat logs`.

  Automatic refresh requires a running daemon (`xrat daemon install --start`).
  Manual refresh uses the exact same import + reconciliation path.

## Metadata Extraction

For subscription URLs, xrat extracts metadata from HTTP headers:

```
subscription-userinfo: upload=1024; download=2048; total=10240; expire=1234567890
```

Parsed fields:

- `upload` — bytes uploaded
- `download` — bytes downloaded
- `total` — total quota
- `expire` — expiration timestamp (Unix epoch)

## Error Handling

xrat continues parsing even when individual lines fail:

```
Import Summary
─────────────────────────────────
Source:     https://example.com/sub.txt
Parsed:     45 nodes
Failed:     3 lines
Duplicates: 12 skipped
New:        33 configs added
```

Failed lines are logged with line numbers and error messages.

## Related

- [`import` CLI](../02-cli/import.md) — command reference
- [Deduplication](deduplication.md) — how duplicates are detected
- [Protocols](../05-reference/protocols.md) — supported protocol formats
