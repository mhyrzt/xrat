# Xray Configuration Types

This module provides Rust types for Xray-core configuration, based on the
official [xtls.github.io](https://xtls.github.io) documentation.

## Features

- **Complete type coverage**: All major Xray configuration objects (log, api,
  dns, routing, policy, inbounds, outbounds, transports)
- **Config parsing modes**:
  - **Strict mode**: Rejects unknown fields (useful for validation)
  - **Lenient mode**: Allows unknown fields (useful for forward compatibility)
  - **Auto mode**: Currently behaves like lenient mode
- **Serde integration**: Full serialization and deserialization support
- **Type safety**: Enums for protocol types, networks, security modes, etc.

## Usage

### Basic Parsing

```rust
use xrat::config::xray::XrayConfig;

// Parse in lenient/loose mode (allows unknown fields)
let config = XrayConfig::from_json_loose(json_str)?;

// Parse in strict mode (rejects unknown fields)
let config = XrayConfig::from_json_strict(json_str)?;

// Serialize back to JSON
let json = config.to_json()?;
```

### Using ParseMode

```rust
use xrat::config::xray::{XrayConfig, ParseMode};

let mode = ParseMode::Strict;
let config = XrayConfig::from_json_with_mode(json_str, mode)?;
```

`ParseMode` controls Xray JSON schema tolerance only. It does not change
subscription/share-link import behavior.

### Example Configuration

```rust
use xrat::config::xray::*;

let json = r#"{
  "log": {
    "access": "/var/log/xray/access.log",
    "error": "/var/log/xray/error.log",
    "loglevel": "warning"
  },
  "dns": {
    "servers": ["8.8.8.8", "1.1.1.1"],
    "queryStrategy": "UseIPv4"
  },
  "routing": {
    "domainStrategy": "AsIs",
    "rules": [
      {
        "domain": ["google.com"],
        "outboundTag": "direct"
      }
    ]
  },
  "inbounds": [
    {
      "protocol": "vless",
      "port": 443,
      "settings": {
        "clients": [
          {
            "id": "uuid-here",
            "email": "user@example.com"
          }
        ]
      },
      "streamSettings": {
        "network": "ws",
        "security": "tls",
        "tlsSettings": {
          "serverName": "example.com",
          "certificates": [
            {
              "certificateFile": "/path/to/cert.pem",
              "keyFile": "/path/to/key.pem"
            }
          ]
        }
      }
    }
  ],
  "outbounds": [
    {
      "protocol": "freedom",
      "tag": "direct"
    }
  ]
}"#;

let config = XrayConfig::from_json_loose(json).unwrap();
```

## Type Structure

### Core Types

- `XrayConfig`: Root configuration object
- `LogObject`: Logging configuration
- `ApiObject`: API server configuration
- `DnsObject`: DNS configuration with servers and hosts
- `RoutingObject`: Routing rules and balancers
- `PolicyObject`: Connection policies and limits

### Protocol Types

- `InboundObject`: Inbound protocol configurations (VLESS, VMess, Trojan,
  Shadowsocks, etc.)
- `OutboundObject`: Outbound protocol configurations
- Protocol-specific settings for each protocol type

### Transport Types

- `StreamSettingsObject`: Transport layer configuration
- `TlsObject`: TLS settings
- `RealityObject`: Reality protocol settings
- `WebSocketObject`, `GrpcObject`, `HttpUpgradeObject`, etc.

### Shared Types

- `Network`: TCP, UDP, or both
- `StreamNetwork`: Transport protocols (ws, grpc, http, etc.)
- `Security`: none, tls, reality
- `LogLevel`: debug, info, warning, error, none
- `QueryStrategy`: DNS query strategies
- `DomainStrategy`: Domain resolution strategies
- `PortValue`: Single port or port range

## Parsing Modes

### Strict Mode

Strict mode uses `#[serde(deny_unknown_fields)]` to reject any JSON fields that
don't match the defined types. This is useful for:

- Configuration validation
- Catching typos in field names
- Ensuring compatibility with a specific Xray version

### Lenient Mode

Lenient mode allows unknown fields to be silently ignored. This is useful for:

- Forward compatibility with newer Xray versions
- Working with configurations that have custom extensions
- Partial configuration parsing
