# serve

Start the local HTTP API server.

```bash
xrat serve [flags]
```

## Flags

| Flag            | Description                 |
| --------------- | --------------------------- |
| `--host <host>` | Override HTTP API bind host |
| `--port <port>` | Override HTTP API bind port |

## Examples

Start with default settings (from config.toml):

```bash
xrat serve
```

Override host and port:

```bash
xrat serve --host 0.0.0.0 --port 9090
```

## Configuration

The HTTP API server is configured in `config.toml`:

```toml
[server]
enabled = false
host = "127.0.0.1"
port = 8080
key = { env = "XRAT_API_KEY" }
```

| Field     | Description                          |
| --------- | ------------------------------------ |
| `enabled` | Enable daemon-hosted API (see below) |
| `host`    | Bind host (default: `127.0.0.1`)     |
| `port`    | Bind port (default: `8080`)          |
| `key`     | Optional API key for authentication  |

## Operating Modes

### Foreground Mode

Run `xrat serve` to start the API server in the foreground. Useful for
development or standalone deployment.

### Daemon-Hosted Mode

When `enabled = true` in config.toml, the daemon automatically starts the HTTP
API alongside IPC. The API runs in the same process as the daemon.

### systemd Service

Run as a systemd user service:

```ini
[Unit]
Description=xrat HTTP API
After=network.target

[Service]
ExecStart=/usr/local/bin/xrat serve
Restart=on-failure
Environment=RUST_LOG=info

[Install]
WantedBy=default.target
```

## API Routes

| Route           | Method | Description                                         |
| --------------- | ------ | --------------------------------------------------- |
| `/health`       | GET    | Health check (no auth required)                     |
| `/json`         | GET    | List configs with latest test results as JSON array |
| `/b64`          | GET    | Base64-encoded subscription text payload            |
| `/configs`      | GET    | Paginated config list with details                  |
| `/configs/{id}` | GET    | Single config detail with latest test results       |

### Query Parameters

#### `/json`

| Parameter  | Description                                                 |
| ---------- | ----------------------------------------------------------- |
| `key`      | API key (if authentication is enabled)                      |
| `top`      | Return top N configs sorted by real-delay                   |
| `enabled`  | Filter: `true` for enabled configs only                     |
| `protocol` | Filter by protocol: `vless`, `vmess`, `ss`, `trojan`, `hy2` |

#### `/b64`

| Parameter | Description                            |
| --------- | -------------------------------------- |
| `key`     | API key (if authentication is enabled) |

#### `/configs`

| Parameter  | Description                              |
| ---------- | ---------------------------------------- |
| `key`      | API key (if authentication is enabled)   |
| `page`     | Page number (default: `1`)               |
| `per_page` | Items per page (default: `20`)           |
| `enabled`  | Filter: `true` for enabled configs only  |
| `protocol` | Filter by protocol                       |

#### `/configs/{id}`

| Parameter | Description                            |
| --------- | -------------------------------------- |
| `key`     | API key (if authentication is enabled) |

## Authentication

If `key` is set in config.toml, all routes except `/health` require the `key`
query parameter:

```bash
curl "http://localhost:8080/json?key=secret"
```

```bash
curl "http://localhost:8080/configs?key=secret&page=1&per_page=10"
```

## Response Formats

### `/health`

```json
{
  "status": "ok"
}
```

### `/json`

```json
[
  {
    "id": 42,
    "protocol": "vless",
    "address": "example.com",
    "port": 443,
    "name": "My Node",
    "is_enabled": true,
    "is_active": false,
    "latest_test": {
      "icmp_ok": true,
      "icmp_ms": 15,
      "tcp_ok": true,
      "tcp_ms": 12,
      "real_delay_ok": true,
      "real_delay_ms": 145,
      "download_mbps": null,
      "tested_at": "2026-05-28T10:00:00Z"
    }
  }
]
```

### `/b64`

Returns base64-encoded subscription text compatible with v2rayN, Clash, and
other clients:

```
dmxlc3M6Ly91dWlkQGV4YW1wbGUuY29tOjQ0Mz90eXBlPXdzJnNlY3VyaXR5PXRscyNNeSBOb2RlCnZtZXNzOi8v...
```

### `/configs`

```json
{
  "page": 1,
  "per_page": 20,
  "total": 150,
  "configs": [
    {
      "id": 42,
      "protocol": "vless",
      "address": "example.com",
      "port": 443,
      "name": "My Node",
      "is_enabled": true,
      "is_active": false,
      "subscription_id": 1,
      "created_at": "2026-05-20T08:00:00Z",
      "updated_at": "2026-05-28T10:00:00Z",
      "latest_test": { ... }
    }
  ]
}
```

### `/configs/{id}`

```json
{
  "id": 42,
  "protocol": "vless",
  "address": "example.com",
  "port": 443,
  "uuid": "uuid-123",
  "network": "ws",
  "tls": "tls",
  "sni": "cdn.example.com",
  "host": "cdn.example.com",
  "path": "/ray",
  "name": "My Node",
  "is_enabled": true,
  "is_active": false,
  "subscription_id": 1,
  "created_at": "2026-05-20T08:00:00Z",
  "updated_at": "2026-05-28T10:00:00Z",
  "latest_test": { ... }
}
```

## Use Cases

- **Subscription server**: Serve configs to mobile/desktop clients via `/b64`
- **Monitoring**: Poll `/health` and `/configs` for uptime monitoring
- **Integration**: Build dashboards or automation around `/json` and `/configs`
- **Proxy management**: Query active configs and test results programmatically

## Related

- [`daemon`](daemon.md) — daemon-hosted API mode
- [`test`](test.md) — test results are exposed via the API
- [`list`](list.md) — CLI equivalent of `/configs`
