# Features

xrat provides a comprehensive set of features for managing proxy configurations
and running local proxy services.

## Core Features

| Feature                                     | Description                                                   |
| ------------------------------------------- | ------------------------------------------------------------- |
| [Importing](importing.md)                   | Import subscriptions from URLs, files, raw text, base64, JSON |
| [Testing](testing.md)                       | 5-stage probe pipeline with failure classification            |
| [Runtime Management](runtime-management.md) | Connect lifecycle, session state, reattach                    |
| [Daemon and IPC](daemon-and-ipc.md)         | Supervisor process with Unix socket IPC                       |
| [Auto-Rotation](auto-rotation.md)           | Scheduled proxy switching with cooldown                       |
| [IP Scanning](ip-scanning.md)               | TCP reachability scanning with persistence                    |
| [HTTP API](http-api.md)                     | RESTful API for config access and monitoring                  |
| [Deduplication](deduplication.md)           | Versioned dedup keys for config uniqueness                    |

## Feature Highlights

### Multi-Protocol Support

xrat supports 7 proxy protocols:

- **VLESS** — modern, lightweight protocol
- **VMess** — legacy protocol with encryption
- **Shadowsocks** — simple SOCKS5-like proxy
- **Trojan** — TLS-based proxy that mimics HTTPS
- **HTTP/HTTPS** — standard HTTP proxy
- **SOCKS5** — classic SOCKS protocol
- **Hysteria2** — QUIC-based protocol (via sing-box)

### Dual Database Backend

- **SQLite** — single-user, file-based, zero configuration
- **PostgreSQL** — multi-user, connection pooling, production-ready

### Engine Support

- **Xray-core/V2Ray** — managed runtime engines for supported Xray/V2Ray
  protocols
- **sing-box** — sing-box JSON preview plus managed Hysteria2 runtime sessions
  through `xrat connect`

### Configurable Testing Pipeline

- 5 test stages: ICMP, TCP, real-delay, download, upload
- Configurable stage order and failure policy
- Bulk testing with concurrency control
- Failure classification with 10 categories
- GeoIP enrichment for endpoint metadata

### Managed Runtime

- Automatic proxy process lifecycle management
- Session state tracking with database persistence
- Graceful shutdown with SIGTERM/SIGKILL fallback
- Stale session recovery on daemon restart

### Daemon Supervisor

- Long-lived background process
- Unix domain socket IPC for CLI communication
- Health monitoring with automatic rotation on failure
- Scheduled rotation with cooldown protection

### HTTP API

- RESTful endpoints for config access
- Base64 subscription output for mobile clients
- Optional API key authentication
- Paginated config listing with filters

## Architecture

See [Architecture](../06-architecture/index.html) for details on how these
features are implemented.
