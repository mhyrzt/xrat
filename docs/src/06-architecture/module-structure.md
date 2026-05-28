# Module Structure

xrat follows a modular architecture with clear separation of concerns across CLI
parsing, command handlers, config parsing, database access, and engine
integration.

## Source Tree

```
src/
├── main.rs           # Entrypoint: parse CLI, init tracing, dispatch command
├── lib.rs            # Re-exports all public modules
│
├── cli/              # Clap command/flag definitions
│   ├── mod.rs        # Module root, pub re-exports
│   ├── root.rs       # Cli struct with global flags
│   ├── command.rs    # Command enum (all subcommands)
│   ├── add.rs        # AddArgs
│   ├── connect.rs    # ConnectArgs
│   ├── daemon.rs     # DaemonArgs + DaemonAction
│   ├── disconnect.rs # DisconnectArgs
│   ├── import.rs     # ImportArgs
│   ├── list.rs       # ListArgs + ListTarget
│   ├── parse.rs      # ParseArgs + ParseEngine
│   ├── proxy.rs      # ProxyArgs + ProxyAction
│   ├── scan.rs       # ScanArgs
│   ├── serve.rs      # ServeArgs
│   ├── status.rs     # StatusArgs
│   ├── test_cmd/     # TestArgs + enums (TestFormat, TestSortBy)
│   └── tests/        # CLI parsing tests
│
├── app/              # Application layer
│   ├── mod.rs
│   ├── context.rs    # AppContext: DB + config + runtime paths
│   ├── context/
│   │   ├── paths.rs  # Runtime path resolution
│   │   └── tests/    # Context tests (binary, database resolution)
│   ├── config/       # AppConfig TOML deserialization
│   ├── error.rs      # AppError enum
│   ├── app_paths/    # Filesystem layout resolution
│   ├── import.rs     # Input loading + b64 detection
│   ├── input/        # Input source reading (URL fetch, file, stdin)
│   ├── commands/     # Command handlers
│   │   ├── mod.rs    # Command dispatch
│   │   ├── add.rs
│   │   ├── connect.rs
│   │   ├── daemon.rs
│   │   ├── disconnect.rs
│   │   ├── import.rs
│   │   ├── list.rs
│   │   ├── parse.rs
│   │   ├── proxy.rs
│   │   ├── scan.rs
│   │   ├── serve.rs
│   │   ├── test.rs
│   │   ├── status/
│   │   └── runtime_output.rs
│   ├── runtime_service/  # Proxy process lifecycle
│   │   ├── connect/      # Connect flow
│   │   ├── replace_flow/ # Atomic disconnect + connect
│   │   ├── reattach/     # Stale session recovery
│   │   ├── session_state/# State transitions + inbound health
│   │   ├── status.rs     # Runtime status snapshot
│   │   └── tests/        # Integration tests
│   └── daemon/       # Daemon supervisor
│       ├── ipc/      # Unix socket IPC protocol
│       │   ├── types.rs      # Request/response types
│       │   ├── handler/      # IPC request dispatch
│       │   ├── client/       # IPC client (Unix + unsupported)
│       │   ├── responses/    # Response builders
│       │   ├── transport/    # Wire format (JSON)
│       │   └── tests/        # IPC integration tests
│       └── supervisor/      # Event loop
│           ├── types.rs     # SupervisorState
│           └── handlers/    # Health check, rotation, runtime
│
├── model/             # Shared domain types
│   ├── node.rs        # Node struct
│   ├── protocol.rs    # Protocol enum
│   └── node_dedup_key.rs  # Dedup key generation
│
├── config/            # Config parsing and normalization
│   ├── protocols/     # Protocol-specific parsers
│   │   ├── vless.rs   # vless:// parser
│   │   ├── vmess.rs   # vmess:// parser
│   │   ├── ss.rs      # ss:// parser
│   │   ├── trojan.rs  # trojan:// parser
│   │   ├── http.rs    # http:// parser
│   │   ├── socks5.rs  # socks5:// parser
│   │   └── hy2.rs     # hysteria2:// parser
│   ├── line.rs        # Line-by-line text parsing
│   ├── normalize.rs   # Node normalization defaults
│   ├── parse_service.rs # Engine-aware parsing
│   ├── import/        # Import format detection
│   │   ├── detect.rs  # Format detection heuristics
│   │   ├── parsers/   # Format-specific parsers
│   │   └── subscription.rs # URL fetch + metadata
│   └── parsing_helpers.rs # Shared URI helpers
│
├── db/                # Database layer
│   ├── connection.rs  # Connection pool management
│   ├── schema.rs      # Migration runner
│   ├── error.rs       # DbError enum
│   ├── database/      # Database query methods
│   ├── repository/    # SQL implementations
│   │   ├── configs/   # Config CRUD
│   │   ├── connection_tests/ # Test result CRUD
│   │   ├── runtime_sessions/ # Session CRUD
│   │   ├── cf_scan_results/  # Scan result CRUD
│   │   ├── subscriptions/    # Subscription CRUD
│   │   └── api/       # API-specific queries
│   └── record/        # Record types (DTOs)
│
├── xray/              # Xray-core integration
│   ├── config/        # Config generation
│   │   ├── generator/ # Probe + runtime config builders
│   │   ├── outbound.rs # Protocol-to-outbound mapping
│   │   ├── stream.rs  # Stream settings (TLS, WS, gRPC, TCP)
│   │   └── types.rs   # XrayConfig, Inbound, Outbound structs
│   ├── parsing/       # Xray JSON config parsing
│   │   ├── core/      # Top-level config structure
│   │   ├── protocols/ # Inbound/outbound protocol parsers
│   │   ├── transports/ # Transport settings parsers
│   │   └── shared/    # Shared types (enums, strings)
│   ├── process/       # Low-level process spawn + lifecycle
│   └── process_mgmt/  # High-level process management + signals
│
├── singbox/           # sing-box integration
│   ├── config/        # sing-box config generation (hy2)
│   └── process_mgmt/  # sing-box process management
│
├── prober/            # Connection testing probes
│   ├── icmp/          # ICMP ping (parse system ping output)
│   ├── tcp/           # TCP connectivity check + failure classification
│   ├── real_delay/    # HTTP round-trip latency via proxy
│   ├── download/      # Download speed measurement
│   └── upload/        # Upload speed measurement
│
├── server/            # Axum HTTP API
│   ├── routes/        # Route handlers (health, json, b64, configs)
│   ├── auth.rs        # API key authentication
│   ├── response.rs    # Response types
│   ├── state.rs       # ServerState
│   └── error.rs       # Server error types
│
└── support/           # Shared utilities
    ├── decode.rs      # Base64 decoding
    ├── geoip.rs       # MaxMind GeoIP lookups
    ├── net.rs         # Network utilities
    ├── time.rs        # Timestamp helpers
    └── url.rs         # URL detection helpers
```

## Module Responsibilities

| Module     | Responsibility                                                                        |
| ---------- | ------------------------------------------------------------------------------------- |
| `cli/`     | Define CLI interface with Clap. Parse args and flags. Test parsing.                   |
| `app/`     | Orchestrate command execution. Manage app lifecycle (context, config, daemon).        |
| `model/`   | Shared domain types (Node, Protocol, NodeDedupKey). No dependencies on other modules. |
| `config/`  | Parse proxy URIs. Normalize nodes. Detect import formats.                             |
| `db/`      | Database connection, migrations, queries, repositories.                               |
| `xray/`    | Generate Xray JSON configs. Parse Xray JSON. Manage Xray processes.                   |
| `singbox/` | Generate sing-box JSON configs. Manage sing-box processes.                            |
| `prober/`  | Connection testing probes: ICMP, TCP, HTTP real-delay, download, upload.              |
| `server/`  | HTTP API server using Axum. Auth, routes, response types.                             |
| `support/` | Shared utilities: base64 decode, GeoIP, network helpers.                              |

## Data Flow

### Import Flow

```
Input (URL/file/text)
    → app/input/ read source
    → config/import/ detect format
    → config/import/parsers/ parse format
    → config/protocols/ parse individual links
    → config/normalize/ apply defaults
    → model/node_dedup_key/ generate dedup key
    → db/repository/configs/ persist with dedup check
    → db/repository/subscriptions/ create/update subscription
```

### Test Flow

```
CLI args
    → app/commands/test/ resolve settings
    → db/repository/configs/ load configs
    → For each config:
        → xray/config/generator/ generate probe config
        → xray/process/ spawn Xray probe
        → prober/icmp/ ICMP ping
        → prober/tcp/ TCP connect
        → prober/real_delay/ HTTP through proxy
        → prober/download/ download through proxy
        → prober/upload/ upload through proxy
        → xray/process/ kill probe
    → db/repository/connection_tests/ persist results
    → app/commands/test/output/ format and print
```

### Connect Flow

```
CLI args
    → app/commands/connect/ load config
    → app/runtime_service/connect/ start session
    → xray/config/generator/ generate runtime config
    → xray/process_mgmt/ spawn detached process
    → db/repository/runtime_sessions/ persist session
    → Return status
```

### Daemon Flow

```
xrat daemon start
    → app/daemon/supervisor/ event loop
    → app/runtime_service/reattach/ reconcile stale sessions
    → tokio::select! loop:
        → health check (every 15s)
        → IPC events (via Unix socket)
        → rotation timer (if enabled)
```

## Dependency Graph

```
support/ ──> model/ ──> config/ ──> db/
   │                    │             │
   │                    v             v
   │               xray/ ───> prober/
   │                    │        │
   │                    v        v
   │               singbox/  app/
   │                           │
   v                           v
server/                   cli/ ──> main.rs
```

## File Conventions

- **`mod.rs`**: Module root, pub re-exports
- **Names**: Snake_case for files/modules, PascalCase for types, snake_case for
  functions
- **Tests**: `#[cfg(test)] mod tests { ... }` in same file or `tests/` submodule
- **Records/DTOs**: In `db/record/` — thin structs matching DB rows
- **Repository**: In `db/repository/` — SQL query functions separated by entity
