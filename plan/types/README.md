# Types Structure

This directory contains TypeScript type definitions organized by functionality,
mirroring the `config/` folder structure.

## Directory Structure

```
types-new/
├── shared.ts              # Common primitive types and enums
├── protocols/             # Protocol-specific types (inbounds/outbounds)
│   ├── common.ts          # Shared protocol types (accounts, fallbacks)
│   ├── clients.ts         # Client objects for all protocols
│   ├── inbound-settings.ts   # Settings for each inbound protocol
│   ├── outbound-settings.ts  # Settings for each outbound protocol
│   ├── inbounds.ts        # Inbound protocol objects
│   └── outbounds.ts       # Outbound protocol objects
├── transports/            # Transport layer types
│   ├── security.ts        # TLS, Reality, Sockopt
│   ├── raw.ts             # Raw transport (TCP with headers)
│   ├── kcp.ts             # mKCP transport
│   ├── grpc.ts            # gRPC transport
│   ├── websocket.ts       # WebSocket transport
│   ├── httpupgrade.ts     # HTTP Upgrade transport
│   ├── hysteria.ts        # Hysteria transport
│   ├── xhttp.ts           # XHTTP transport
│   └── finalmask.ts       # Final mask configurations
└── core/                  # Core Xray configuration types
    ├── log.ts             # Logging configuration
    ├── api.ts             # API configuration
    ├── dns.ts             # DNS configuration
    ├── routing.ts         # Routing rules and balancers
    ├── policy.ts          # Policy configuration
    ├── features.ts        # Additional features (stats, reverse, etc.)
    └── config.ts          # Main XrayConfig interface
```

## Key Improvements

1. **Better Organization**: Types are now grouped by functionality matching the
   `config/` folder structure
2. **Deduplication**: Common types like `HttpAccountObject`, `FallbackObject`,
   and `ReverseTagObject` are extracted to `protocols/common.ts`
3. **Separation of Concerns**:
   - Protocol settings are separated from protocol objects
   - Transport types are split by transport type
   - Core configuration types are organized by feature
4. **Easier Navigation**: Each subdirectory has an `index.ts` that re-exports
   all types
5. **Maintainability**: Changes to specific protocols or transports are now
   isolated to their respective files

## Usage

Import types from the main index:

```typescript
import type { XrayConfig, InboundObject, OutboundObject } from "./types-new";
```

Or import from specific modules:

```typescript
import type { VlessClientObject } from "./types-new/protocols/clients";
import type { TLSObject } from "./types-new/transports/security";
import type { RoutingObject } from "./types-new/core/routing";
```
