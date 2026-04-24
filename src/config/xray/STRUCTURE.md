# Xray Module Structure

This document describes the organization of the Xray configuration types module.

## Directory Structure

```txt
src/config/xray/
├── README.md              # Usage documentation
├── STRUCTURE.md           # This file
├── mod.rs                 # Module exports
├── shared.rs              # Shared types and enums
├── core/                  # Core configuration types
│   ├── mod.rs
│   ├── api.rs            # API server configuration
│   ├── config.rs         # Root XrayConfig type
│   ├── dns.rs            # DNS configuration
│   ├── features.rs       # Stats, metrics, observatory, etc.
│   ├── log.rs            # Logging configuration
│   ├── policy.rs         # Connection policies
│   ├── routing.rs        # Routing rules and balancers
│   └── tests.rs          # Integration tests
├── protocols/             # Protocol-specific types
│   ├── mod.rs
│   ├── common.rs         # Shared protocol types
│   ├── inbounds.rs       # Inbound configurations
│   ├── outbounds.rs      # Outbound configurations
│   └── outbound_settings.rs  # Protocol-specific settings
└── transports/            # Transport layer types
    ├── mod.rs
    ├── security.rs       # TLS, Reality, sockopt
    └── streams.rs        # WebSocket, gRPC, HTTP, etc.
```

## Module Organization

### shared.rs

Contains base types used across all modules:

- Type aliases: `Address`, `Cidr`, `DomainMatcher`, `DurationString`, etc.
- Enums: `Network`, `StreamNetwork`, `Security`, `LogLevel`, `QueryStrategy`,
  `DomainStrategy`
- Variant types: `PortValue`, `Int32Range`

### core/

Core Xray configuration objects:

- **api.rs**: API service configuration (`ApiObject`, `ApiServiceName`)
- **config.rs**: Root configuration type (`XrayConfig`) with strict/loose
  parsing
- **dns.rs**: DNS servers and hosts (`DnsObject`, `DnsServerObject`,
  `DnsHostValue`)
- **features.rs**: Optional features (`VersionObject`, `StatsObject`,
  `ReverseObject`, `FakeDnsObject`, `MetricsObject`, `ObservatoryObject`,
  `BurstObservatoryObject`)
- **log.rs**: Logging configuration (`LogObject`)
- **policy.rs**: Connection policies (`PolicyObject`, `LevelPolicyObject`,
  `SystemPolicyObject`)
- **routing.rs**: Routing rules and load balancing (`RoutingObject`,
  `RuleObject`, `BalancerObject`)
- **tests.rs**: Comprehensive test suite for parsing modes

### protocols/

Protocol-specific configuration:

- **common.rs**: Shared protocol types (`FragmentObject`, `NoiseObject`,
  `WireguardPeerObject`, `VlessReverse`)
- **inbounds.rs**: Inbound listener configuration (`InboundObject`,
  `SniffingObject`)
- **outbounds.rs**: Outbound proxy configuration (`OutboundObject` enum with
  variants for each protocol, `MuxObject`, `ProxySettingsObject`)
- **outbound_settings.rs**: Protocol-specific settings for each outbound type
  (Blackhole, DNS, Freedom, HTTP, Hysteria, Loopback, Shadowsocks, Socks,
  Trojan, VLESS, VMess, Wireguard)

### transports/

Transport layer configuration:

- **security.rs**: Security and socket options (`TlsObject`, `RealityObject`,
  `SockoptObject`, `TlsCertificateObject`, `HappyEyeballsObject`,
  `CustomSockoptObject`)
- **streams.rs**: Stream transport types (`StreamSettingsObject`,
  `WebSocketObject`, `GrpcObject`, `HttpUpgradeObject`, `KcpObject`,
  `RawObject`, `HysteriaObject`, `XHttpSettingsObject`, `FinalMaskObject`,
  `TransportObject`)

## Design Principles

1. **Focused files**: Each file contains related types (typically 50-200 lines)
2. **Clear separation**: Core, protocols, and transports are in separate
   directories
3. **TypeScript alignment**: Structure mirrors the TypeScript types from
   xtls.github.io
4. **Minimal dependencies**: Each module imports only what it needs
5. **Testability**: Tests are in a separate file but within the core module

## Parsing Modes

The module supports two parsing modes:

- **Strict mode** (`XrayConfig::from_json_strict`): Rejects unknown fields using
  `#[serde(deny_unknown_fields)]`
- **Loose mode** (`XrayConfig::from_json_loose`): Allows unknown fields for
  forward compatibility

See `README.md` for usage examples.
