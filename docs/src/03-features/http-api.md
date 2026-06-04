# HTTP API

xrat includes an Axum-based HTTP API server that exposes stored configs, test
results, and subscription-compatible output for integration with external tools
and clients.

## Overview

The HTTP API provides:

- **Health check** — verify server is running
- **Config listing** — query configs with filters and pagination
- **Subscription output** — base64-encoded subscription text for mobile clients
- **JSON export** — machine-readable config data with test results

## Starting the Server

### Foreground Mode

```bash
xrat serve
```

Starts the server in the foreground using settings from `config.toml`.

### Override Host and Port

```bash
xrat serve --host 0.0.0.0 --port 9090
```

### Daemon-Hosted Mode

When `enabled = true` in config.toml, the daemon automatically starts the HTTP
API alongside IPC:

```toml
[server]
enabled = true
host = "127.0.0.1"
port = 8080
```

## Configuration

```toml
[server]
enabled = false
host = "127.0.0.1"
port = 8080
key = { env = "XRAT_API_KEY" }
```

| Field     | Description                         | Default     |
| --------- | ----------------------------------- | ----------- |
| `enabled` | Enable daemon-hosted API            | `false`     |
| `host`    | Bind host                           | `127.0.0.1` |
| `port`    | Bind port                           | `8080`      |
| `key`     | Optional API key for authentication | -           |

## Routes

| Route           | Method | Description                | Auth Required    |
| --------------- | ------ | -------------------------- | ---------------- |
| `/health`       | GET    | Health check               | No               |
| `/json`         | GET    | List configs as JSON array | Yes (if key set) |
| `/b64`          | GET    | Base64 subscription text   | Yes (if key set) |
| `/configs`      | GET    | Paginated config list      | Yes (if key set) |
| `/configs/{id}` | GET    | Single config detail       | Yes (if key set) |

## Authentication

If `key` is set in config.toml, all routes except `/health` require the `key`
query parameter:

```bash
curl "http://localhost:8080/json?key=secret"
```

The key can be a literal string or an environment variable:

```toml
key = "literal-secret"
key = { env = "XRAT_API_KEY" }
```

## Route Details

### GET /health

Health check endpoint (no authentication required).

**Request**:

```bash
curl http://localhost:8080/health
```

**Response**:

```json
{
  "status": "ok"
}
```

**Status**: `200 OK`

---

### GET /json

List configs with latest test results as a JSON array.

**Query Parameters**:

| Parameter  | Type    | Description                                                 |
| ---------- | ------- | ----------------------------------------------------------- |
| `key`      | string  | API key (if authentication enabled)                         |
| `top`      | integer | Return top N configs sorted by real-delay                   |
| `enabled`  | boolean | Filter: `true` for enabled configs only                     |
| `protocol` | string  | Filter by protocol: `vless`, `vmess`, `ss`, `trojan`, `hy2` |

**Request**:

```bash
curl "http://localhost:8080/json?key=secret&top=10&enabled=true"
```

**Response**:

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
  },
  {
    "id": 43,
    "protocol": "vmess",
    "address": "edge.com",
    "port": 8443,
    "name": "Edge Node",
    "is_enabled": true,
    "is_active": false,
    "latest_test": {
      "icmp_ok": true,
      "icmp_ms": 18,
      "tcp_ok": true,
      "tcp_ms": 14,
      "real_delay_ok": true,
      "real_delay_ms": 162,
      "download_mbps": null,
      "tested_at": "2026-05-28T10:00:00Z"
    }
  }
]
```

**Status**: `200 OK`

---

### GET /b64

Base64-encoded subscription text compatible with v2rayN, Clash, and other
clients.

**Query Parameters**:

| Parameter | Type   | Description                         |
| --------- | ------ | ----------------------------------- |
| `key`     | string | API key (if authentication enabled) |

**Request**:

```bash
curl "http://localhost:8080/b64?key=secret"
```

**Response**:

```
dmxlc3M6Ly91dWlkQGV4YW1wbGUuY29tOjQ0Mz90eXBlPXdzJnNlY3VyaXR5PXRscyNNeSBOb2RlCnZtZXNzOi8v...
```

**Status**: `200 OK`

**Content-Type**: `text/plain`

The response is a base64-encoded string containing one share link per line:

```
vless://uuid@example.com:443?type=ws&security=tls#My Node
vmess://eyJhZGQiOiJlZGdlLmNvbSIsInBvcnQiOiI4NDQzIn0=#Edge Node
```

---

### GET /configs

Paginated config list with details.

**Query Parameters**:

| Parameter  | Type    | Description                             | Default |
| ---------- | ------- | --------------------------------------- | ------- |
| `key`      | string  | API key (if authentication enabled)     | -       |
| `page`     | integer | Page number                             | `1`     |
| `per_page` | integer | Items per page                          | `20`    |
| `enabled`  | boolean | Filter: `true` for enabled configs only | -       |
| `protocol` | string  | Filter by protocol                      | -       |

**Request**:

```bash
curl "http://localhost:8080/configs?key=secret&page=1&per_page=10&enabled=true"
```

**Response**:

```json
{
  "page": 1,
  "per_page": 10,
  "total": 150,
  "configs": [
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
}
```

**Status**: `200 OK`

---

### GET /configs/{id}

Single config detail with latest test results.

**Path Parameters**:

| Parameter | Type    | Description |
| --------- | ------- | ----------- |
| `id`      | integer | Config ID   |

**Query Parameters**:

| Parameter | Type   | Description                         |
| --------- | ------ | ----------------------------------- |
| `key`     | string | API key (if authentication enabled) |

**Request**:

```bash
curl "http://localhost:8080/configs/42?key=secret"
```

**Response**:

```json
{
  "id": 42,
  "protocol": "vless",
  "address": "example.com",
  "port": 443,
  "uuid": "uuid-123",
  "password": null,
  "method": null,
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
  "latest_test": {
    "icmp_ok": true,
    "icmp_ms": 15,
    "tcp_ok": true,
    "tcp_ms": 12,
    "real_delay_ok": true,
    "real_delay_ms": 145,
    "connect_ms": 10,
    "ttfb_ms": 120,
    "http_status": 204,
    "download_mbps": null,
    "upload_mbps": null,
    "failure_kind": null,
    "failure_reason": null,
    "endpoint_ip": "93.184.216.34",
    "endpoint_country": "US",
    "endpoint_asn": "AS15133",
    "tested_at": "2026-05-28T10:00:00Z"
  }
}
```

**Status**: `200 OK`

**Error Response** (config not found):

```json
{
  "error": "config not found",
  "id": 999
}
```

**Status**: `404 Not Found`

## Error Responses

### Authentication Failed

```json
{
  "error": "unauthorized"
}
```

**Status**: `401 Unauthorized`

### Not Found

```json
{
  "error": "config not found",
  "id": 999
}
```

**Status**: `404 Not Found`

### Internal Error

```json
{
  "error": "internal server error"
}
```

**Status**: `500 Internal Server Error`

## Use Cases

### Subscription Server

Serve configs to mobile/desktop clients:

```bash
# v2rayN / Clash
curl "http://localhost:8080/b64?key=secret" > subscription.txt
```

Configure clients to fetch from `http://your-server:8080/b64?key=secret`.

### Monitoring

Poll health and config status:

```bash
# Health check
curl http://localhost:8080/health

# Active configs
curl "http://localhost:8080/configs?key=secret&enabled=true"
```

### Dashboard Integration

Build a web dashboard that queries the API:

```javascript
fetch("http://localhost:8080/configs?key=secret&page=1&per_page=20")
  .then((r) => r.json())
  .then((data) => {
    console.log(`Total configs: ${data.total}`);
    data.configs.forEach((c) => {
      console.log(`${c.name}: ${c.latest_test?.real_delay_ms}ms`);
    });
  });
```

### Automation

Script proxy management:

```bash
# Get top 5 configs by latency
TOP=$(curl -s "http://localhost:8080/json?key=secret&top=5&enabled=true")

# Extract config IDs
IDS=$(echo "$TOP" | jq -r '.[].id')

# Test each config
for id in $IDS; do
  xrat test $id
done
```

## Security Considerations

### Bind Address

By default, the server binds to `127.0.0.1` (localhost only). To expose
externally:

```toml
[server]
host = "0.0.0.0"
```

**Warning**: Only expose externally if authentication is enabled and the network
is trusted.

### API Key

Use a strong, random API key:

```bash
# Generate a random key
openssl rand -hex 32

# Set in config.toml
key = "a1b2c3d4e5f6..."
```

Or use an environment variable:

```toml
key = { env = "XRAT_API_KEY" }
```

```bash
export XRAT_API_KEY=$(openssl rand -hex 32)
xrat serve
```

### HTTPS

The server does not support HTTPS natively. Use a reverse proxy (nginx, Caddy)
for TLS termination:

```nginx
server {
    listen 443 ssl;
    server_name xrat.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
    }
}
```

## systemd Service

Run as a systemd user service:

```ini
[Unit]
Description=xrat HTTP API
After=network.target

[Service]
ExecStart=/usr/local/bin/xrat serve
Restart=on-failure
Environment=RUST_LOG=info
Environment=XRAT_API_KEY=your-secret-key

[Install]
WantedBy=default.target
```

Enable and start:

```bash
systemctl --user daemon-reload
systemctl --user enable xrat-api
systemctl --user start xrat-api
```

## Related

- [`serve` CLI](../02-cli/serve.md) — command reference
- [Daemon and IPC](daemon-and-ipc.md) — daemon-hosted API mode
- [Deployment](../04-deployment/systemd.md) — systemd service examples
