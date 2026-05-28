# Error Codes

xrat categorizes errors into application-level errors and test failure
classifications.

## AppError

`AppError` is the primary error type returned by command handlers and services.

| Variant | Description | Use Case |
|---------|-------------|----------|
| `ConfigNotFound` | Config ID not found in database | `xrat connect <id>` with invalid ID |
| `NoActiveSession` | No active proxy session | `xrat disconnect` with no session |
| `XraySpawn` | Failed to spawn Xray process | Xray binary not found or invalid |
| `XrayExited` | Xray process exited unexpectedly | Process crashed during startup |
| `XrayStartupTimeout` | Xray port not ready within timeout | Slow startup or port conflict |
| `DaemonNotRunning` | Daemon IPC socket not reachable | `xrat proxy start` without daemon |
| `DaemonConnect` | Failed to connect to daemon socket | Permission denied or socket missing |
| `Database` | Database query or connection error | Connection failure or constraint violation |
| `Io` | Filesystem I/O error | Permission denied or disk full |
| `Config` | Configuration file error | Invalid TOML or missing required field |
| `MissingPostgresUser` | PostgreSQL user not configured | `database.postgres.user` is empty |
| `MissingPostgresDatabaseName` | PostgreSQL database name not configured | `database.postgres.db_name` is empty |
| `InvalidConfigValue` | Invalid configuration value | Unknown enum variant or out-of-range |
| `Serialization` | JSON serialization/deserialization error | Invalid JSON or schema mismatch |
| `Probe` | Probe test execution error | ICMP ping command failed |
| `Parse` | Config link parsing error | Invalid URI format or unsupported scheme |

### Error Messages

Errors implement `Display` for user-friendly messages:

```
Error: config not found (id: 42)
Error: daemon socket not reachable at /home/user/.config/xrat/runtime/daemon.sock
Error: Xray process failed to start: No such file or directory (os error 2)
```

## DbError

`DbError` represents database-specific errors.

| Variant | Description |
|---------|-------------|
| `Query` | SQL query execution error |
| `Pool` | Connection pool acquisition error |
| `Connection` | Database connection error |
| `UniqueViolation` | Duplicate key violation (used for dedup) |
| `ForeignKeyViolation` | Referential integrity violation |
| `NotFound` | Expected row not found |
| `Migration` | Schema migration error |
| `Config` | Database configuration error |

## FailureKind

`FailureKind` classifies test stage failures. Used by the testing pipeline and
displayed in test results.

| Category | Description | Example |
|----------|-------------|---------|
| `DNS` | DNS resolution failed | `nodename nor servname provided, or not known` |
| `Timeout` | Connection or request timed out | `connection timed out after 5000ms` |
| `Refused` | Connection refused | `Connection refused (os error 111)` |
| `Unreachable` | Network unreachable | `No route to host (os error 113)` |
| `PermissionDenied` | Permission denied | `Operation not permitted` |
| `TLS` | TLS handshake failed | `tls: first record does not look like a TLS handshake` |
| `Auth` | Authentication failed | `proxy authentication required` |
| `Process` | Proxy process failed to start | `xray binary not found` |
| `Proxy` | Proxy returned an error status | `HTTP 503 Service Unavailable` |
| `Unknown` | Unclassified failure | Any other error |

### Failure Classification Logic

TCP failures are classified by matching error strings:

```rust
fn classify_tcp_error(error: &io::Error) -> FailureKind {
    match error.kind() {
        io::ErrorKind::ConnectionRefused => FailureKind::Refused,
        io::ErrorKind::ConnectionReset => FailureKind::Refused,
        io::ErrorKind::TimedOut => FailureKind::Timeout,
        io::ErrorKind::ConnectionAborted => FailureKind::Timeout,
        io::ErrorKind::NotConnected => FailureKind::Unreachable,
        io::ErrorKind::AddrInUse => FailureKind::PermissionDenied,
        io::ErrorKind::AddrNotAvailable => FailureKind::PermissionDenied,
        io::ErrorKind::PermissionDenied => FailureKind::PermissionDenied,
        io::ErrorKind::HostUnreachable => FailureKind::Unreachable,
        io::ErrorKind::NetworkUnreachable => FailureKind::Unreachable,
        io::ErrorKind::InvalidInput => FailureKind::Unknown,
        _ => {
            let msg = error.to_string().to_lowercase();
            if msg.contains("dns") || msg.contains("resolve") {
                FailureKind::DNS
            } else {
                FailureKind::Unknown
            }
        }
    }
}
```

## ConfigParseError

`ConfigParseError` is returned by the config parser when parsing share links.

| Variant | Description |
|---------|-------------|
| `Url` | Invalid URL format |
| `Json` | Invalid JSON (vmess://) |
| `Decode` | Invalid base64 payload |
| `ParseInt` | Invalid numeric value |
| `MissingAddressOrPort` | URI missing address or port |
| `MissingBase64Userinfo` | URI missing base64-encoded userinfo |
| `InvalidShadowsocksUserinfo` | Invalid Shadowsocks userinfo format |
| `MissingRequiredField` | Required field not found in JSON |
| `UnsupportedScheme` | Unknown protocol scheme |

## XrayProcessError

`XrayProcessError` is returned by the Xray process manager.

| Variant | Description |
|---------|-------------|
| `TempFileError` | Failed to create temporary config file |
| `SerializationError` | Failed to serialize config JSON |
| `SpawnError` | Failed to spawn Xray process |
| `StartupTimeout` | Xray failed to start within timeout |
| `ProcessExited` | Xray exited unexpectedly (with stderr) |
| `PortNotReady` | Inbound port not ready within timeout |

## ImportParseError

`ImportParseError` is returned by the import parser.

| Variant | Description |
|---------|-------------|
| `InvalidShareLink` | Input is not a valid share link |
| `Decode` | Invalid base64 decoding |
| `Json` | Invalid JSON |
| `MissingSip008Servers` | SIP008 JSON missing `servers` array |
| `MissingSip008Field` | SIP008 server missing required field |
| `Xray` | Invalid Xray JSON |
| `Config` | Invalid config node |

## Error Handling Best Practices

### CLI Commands

Command handlers return `Result<(), AppError>`:

```rust
pub async fn run(context: &AppContext, args: &ConnectArgs) -> Result<()> {
    let config = context.db.get_config(args.id).await?
        .ok_or(AppError::ConfigNotFound(args.id))?;
    // ...
}
```

### Logging

Errors are logged at `error` level before returning:

```rust
if let Err(err) = run(&context, &args.command).await {
    tracing::error!(error = %err, "command failed");
    std::process::exit(1);
}
```

### User-Facing Messages

Errors display actionable messages when possible:

```
Error: daemon socket not reachable
Hint: start the daemon with 'xrat daemon start'
```
