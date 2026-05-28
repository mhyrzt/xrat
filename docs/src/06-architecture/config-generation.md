# Config Generation

xrat generates runtime configuration JSON from normalized `Node` objects for
both Xray-core and sing-box engines.

## Overview

The config generation pipeline:

1. **Node** (domain model) → Protocol-specific mapping → JSON config
2. Supports probe configs (short-lived, for testing) and runtime configs (long-lived)

## Xray Config Generation

Located in `src/xray/config/`.

### Entry Points

```rust
// Probe config (used for testing)
pub fn generate_probe_config(node: &Node, local_port: u16) -> Result<XrayConfig, String>

// Runtime config (used for connect)
pub fn generate_runtime_config(node: &Node, socks_port: u16, http_port: Option<u16>) -> Result<XrayConfig, String>

// Runtime config with custom inbound hosts
pub fn generate_runtime_config_with_inbounds(
    node: &Node,
    socks_host: &str,
    socks_port: u16,
    http_host: Option<&str>,
    http_port: Option<u16>,
) -> Result<XrayConfig, String>

// Runtime config with fully configurable inbounds
pub fn generate_runtime_config_for_inbounds(
    node: &Node,
    socks: Option<(&str, u16, bool)>,
    http: Option<(&str, u16)>,
) -> Result<XrayConfig, String>
```

### Probe Config

Used by the testing pipeline to measure real-delay latency:

```json
{
  "log": { "loglevel": "warning" },
  "inbounds": [
    {
      "tag": "probe-in",
      "port": <random>,
      "listen": "127.0.0.1",
      "protocol": "socks",
      "settings": { "udp": false }
    }
  ],
  "outbounds": [
    {
      "tag": "proxy",
      "protocol": "<node.protocol>",
      "settings": { ... },
      "stream_settings": { ... }
    }
  ]
}
```

### Runtime Config

Used by the managed proxy session:

```json
{
  "log": { "loglevel": "warning" },
  "inbounds": [
    {
      "tag": "socks-in",
      "port": 1080,
      "listen": "0.0.0.0",
      "protocol": "socks",
      "settings": { "udp": true }
    },
    {
      "tag": "http-in",
      "port": 8080,
      "listen": "0.0.0.0",
      "protocol": "http"
    }
  ],
  "outbounds": [
    {
      "tag": "proxy",
      "protocol": "<node.protocol>",
      "settings": { ... },
      "stream_settings": { ... }
    }
  ]
}
```

### XrayConfig Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrayConfig {
    pub log: LogConfig,
    pub inbounds: Vec<Inbound>,
    pub outbounds: Vec<Outbound>,
}
```

### Outbound Generation

Each protocol maps to a specific outbound format:

#### VLESS

```json
{
  "vnext": [{
    "address": "example.com",
    "port": 443,
    "users": [{
      "id": "uuid-123",
      "encryption": "none"
    }]
  }]
}
```

#### VMess

```json
{
  "vnext": [{
    "address": "example.com",
    "port": 443,
    "users": [{
      "id": "uuid-456",
      "alterId": 0,
      "security": "auto"
    }]
  }]
}
```

#### Trojan

```json
{
  "servers": [{
    "address": "example.com",
    "port": 443,
    "password": "password"
  }]
}
```

#### Shadowsocks

```json
{
  "servers": [{
    "address": "example.com",
    "port": 8388,
    "method": "aes-256-gcm",
    "password": "secret"
  }]
}
```

#### SOCKS5

```json
{
  "servers": [{
    "address": "example.com",
    "port": 1080,
    "users": [{
      "user": "username",
      "pass": "password"
    }]
  }]
}
```

#### HTTP

```json
{
  "servers": [{
    "address": "example.com",
    "port": 8080,
    "users": [{
      "user": "username",
      "pass": "password"
    }]
  }]
}
```

#### Hysteria2

Hysteria2 is not supported by Xray. Returns an error if attempted:

```
Error: hysteria2/hy2 is not supported by xray config generator
```

### Stream Settings

Generated based on node fields:

```rust
fn build_stream_settings(node: &Node) -> Result<Option<StreamSettings>, String> {
    let network = node.network.as_str();
    
    // SOCKS5 and HTTP have no stream settings
    if matches!(node.protocol, Protocol::Socks5 | Protocol::Http) {
        return Ok(None);
    }
    
    // Network-specific settings
    StreamSettings {
        network,          // "tcp" | "ws" | "grpc" | "kcp" | "http"
        security,         // "tls" | "none" | None
        tls_settings,     // SNI, allow_insecure
        ws_settings,      // path, headers (Host)
        tcp_settings,     // HTTP header obfuscation
        grpc_settings,    // service_name
    }
}
```

#### TLS Settings

```json
{
  "network": "tcp",
  "security": "tls",
  "tlsSettings": {
    "serverName": "cdn.example.com",
    "allowInsecure": false
  }
}
```

#### WebSocket Settings

```json
{
  "network": "ws",
  "security": "tls",
  "tlsSettings": {
    "serverName": "cdn.example.com"
  },
  "wsSettings": {
    "path": "/ray",
    "headers": {
      "Host": "cdn.example.com"
    }
  }
}
```

#### gRPC Settings

```json
{
  "network": "grpc",
  "grpcSettings": {
    "serviceName": "service"
  }
}
```

#### TCP HTTP Obfuscation

```json
{
  "network": "tcp",
  "tcpSettings": {
    "header": {
      "type": "http",
      "request": {
        "path": ["/custom-path"]
      }
    }
  }
}
```

### Stream Settings Logic

```rust
pub(super) fn build_stream_settings(node: &Node) -> Result<Option<StreamSettings>, String> {
    let network = node.network.as_str();

    if matches!(node.protocol, Protocol::Socks5 | Protocol::Http) {
        return Ok(None);
    }

    let security = node.tls.as_ref().map(|s| s.to_string());
    let tls_settings = if node.tls.as_deref() == Some("tls") {
        Some(TlsSettings {
            server_name: node.sni.clone().unwrap_or_else(|| node.address.clone()),
            allow_insecure: None,
        })
    } else {
        None
    };

    let ws_settings = if network == "ws" {
        let mut headers = HashMap::new();
        if let Some(host) = &node.host {
            headers.insert("Host".to_string(), host.clone());
        }
        Some(WsSettings {
            path: node.path.clone().unwrap_or_else(|| "/".to_string()),
            headers: if headers.is_empty() { None } else { Some(headers) },
        })
    } else {
        None
    };

    let grpc_settings = if network == "grpc" {
        Some(GrpcSettings {
            service_name: node.path.clone().unwrap_or_default(),
        })
    } else {
        None
    };

    let tcp_settings = if network == "tcp" {
        node.path.as_ref().map(|path| TcpSettings {
            header: Some(json!({
                "type": "http",
                "request": { "path": [path] }
            })),
        })
    } else {
        None
    };

    Ok(Some(StreamSettings {
        network: network.to_string(),
        security,
        tls_settings,
        ws_settings,
        tcp_settings,
        grpc_settings,
    }))
}
```

## sing-box Config Generation

Located in `src/singbox/config/`.

### Supported Protocols

Currently only **Hysteria2** is implemented.

### Hysteria2 Config

```rust
pub fn generate_probe_config(node: &Node, local_port: u16) -> Result<Value, String>
pub fn generate_runtime_config(node: &Node, socks_port: u16) -> Result<Value, String>
```

### Probe Config

```json
{
  "log": { "level": "warn" },
  "inbounds": [
    {
      "type": "socks",
      "tag": "socks-in",
      "listen": "127.0.0.1",
      "listen_port": <local_port>
    }
  ],
  "outbounds": [
    {
      "type": "hysteria2",
      "tag": "proxy",
      "server": "example.com",
      "server_port": 443,
      "password": "secret",
      "tls": {
        "enabled": true,
        "server_name": "cdn.example.com"
      }
    },
    {
      "type": "direct",
      "tag": "direct"
    }
  ]
}
```

## Engine Resolution

The parser service resolves which engine to use:

```rust
pub fn resolve_engine(mode: EngineMode, protocol: Protocol) -> Result<ResolvedEngine, ConfigParseError> {
    match mode {
        EngineMode::Auto => {
            if matches!(protocol, Protocol::Hy2) {
                Ok(ResolvedEngine::SingBox)
            } else {
                Ok(ResolvedEngine::Xray)
            }
        }
        EngineMode::Xray => {
            if matches!(protocol, Protocol::Hy2) {
                return Err(ConfigParseError::UnsupportedScheme(
                    "hysteria2/hy2 is not compatible with xray engine".to_string()
                ));
            }
            Ok(ResolvedEngine::Xray)
        }
        EngineMode::SingBox => Ok(ResolvedEngine::SingBox),
    }
}
```

## Xray JSON Parsing

xrat includes a full Xray JSON config parser for reading existing configs.

### Parser Modes

| Mode | Behavior |
|------|----------|
| `strict` | Rejects unknown fields using `#[serde(deny_unknown_fields)]` |
| `lenient` | Allows unknown fields (default) |
| `auto` | Same as lenient (reserved for future source-aware parsing) |

### Parsed Structures

The parser can read:

- **Log**: access, error, loglevel, dns_log, mask_address
- **API**: tag, listen, services
- **DNS**: hosts, servers, client_ip, query_strategy, cache/fallback settings
- **Routing**: domain_strategy, rules, balancers
- **Policy**: levels (handshake, connIdle, etc.), system stats
- **Inbounds**: various protocol inbounds with full settings
- **Outbounds**: various protocol outbounds with full settings
- **Transports**: TCP, WebSocket, gRPC, HTTP/2, QUIC, KCP
- **Features**: stats, reverse, fakedns, metrics, observatory

## Process Management

### Xray Process Spawning

Low-level process lifecycle:

```rust
pub async fn spawn(config: &XrayConfig, startup_timeout: Duration) -> Result<XrayProcess, XrayProcessError>
```

1. Write config JSON to temp file
2. Spawn `xray run -c <config_path>`
3. Poll SOCKS port every 100ms
4. Return when port is ready or timeout

### Managed Process Spawning

High-level process lifecycle for daemon:

```rust
pub async fn spawn_detached(
    binary_path: &Path,
    runtime_dir: &Path,
    session_id: i64,
    config: &XrayConfig,
    ready_host: &str,
    ready_port: u16,
    startup_timeout: Duration,
) -> Result<ManagedXrayProcess, AppError>
```

1. Write config to `runtime_dir/session-<id>.json`
2. Create stdout/stderr log files
3. Spawn detached process
4. Poll for readiness
5. Return ManagedXrayProcess (PID, port, paths)

### Signal Handling

```rust
pub fn terminate_process_gracefully(
    pid: i64,
    timeout: Duration,
) -> Result<TerminationOutcome, AppError>
```

1. Send SIGTERM
2. Poll every 100ms up to timeout
3. If still running, send SIGKILL
4. Return outcome: `Terminated`, `Killed`, `NotRunning`

## Config Generation Tests

```rust
#[test]
fn test_generate_vless_probe_config() {
    let node = Node {
        protocol: Protocol::Vless,
        address: "example.com".to_string(),
        port: 443,
        uuid: Some("test-uuid".to_string()),
        network: "tcp".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("example.com".to_string()),
        // ...
    };

    let config = generate_probe_config(&node, 10808).unwrap();
    assert_eq!(config.inbounds[0].port, 10808);
    assert_eq!(config.outbounds[0].protocol, "vless");
}
```
