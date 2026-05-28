# Deduplication

xrat uses versioned, length-prefixed dedup keys to ensure config uniqueness
while distinguishing between `None` and empty string values.

## Overview

When importing configs, xrat:

1. Generates a dedup key for each node
2. Checks if a config with the same key already exists
3. Skips duplicates, only inserting new configs

This prevents the same proxy from being imported multiple times, even across
different subscriptions.

## Dedup Key Format

The dedup key is a string with the following format:

```
v1|protocol=<len>:<value>|address=<len>:<value>|port=<len>:<value>|...
```

### Structure

- **Version prefix**: `v1` (allows future format changes)
- **Field separator**: `|`
- **Field format**: `<name>=<length>:<value>` or `<name>=-` (for None)

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `protocol` | required | Protocol name (vless, vmess, ss, etc.) |
| `address` | required | Server address |
| `port` | required | Server port |
| `username` | optional | Username (HTTP/SOCKS5) |
| `uuid` | optional | UUID (VLESS/VMess) |
| `password` | optional | Password (Trojan/SS/SOCKS5) |
| `method` | optional | Encryption method (Shadowsocks) |
| `network` | required | Network type (tcp, ws, grpc) |
| `tls` | optional | TLS mode (tls, none) |
| `sni` | optional | SNI hostname |
| `host` | optional | Host header (WebSocket) |
| `path` | optional | Path (WebSocket/gRPC) |

### Length Prefix

Each value is prefixed with its character count:

```
protocol=5:vless
address=11:example.com
port=3:443
```

This prevents ambiguity when values contain the separator character (`|`).

### None vs Empty String

- **None**: Represented as `-`
- **Empty string**: Represented as `0:`

Example:

```
username=-          # None
username=0:         # Empty string
username=4:user     # "user"
```

This distinction is important because some protocols treat None and empty
string differently.

## Example Keys

### VLESS with WebSocket + TLS

```
v1|protocol=5:vless|address=11:example.com|port=3:443|username=-|uuid=8:uuid-123|password=-|method=-|network=2:ws|tls=3:tls|sni=15:cdn.example.com|host=15:cdn.example.com|path=4:/ray
```

### VMess with TCP

```
v1|protocol=5:vmess|address=14:vmess.example.com|port=4:8443|username=-|uuid=8:uuid-456|password=-|method=-|network=3:tcp|tls=3:tls|sni=-|host=-|path=-
```

### Shadowsocks

```
v1|protocol=2:ss|address=11:example.com|port=4:8388|username=-|uuid=-|password=6:secret|method=11:aes-256-gcm|network=3:tcp|tls=-|sni=-|host=-|path=-
```

### Trojan

```
v1|protocol=6:trojan|address=11:example.com|port=3:443|username=-|uuid=-|password=8:password|method=-|network=3:tcp|tls=3:tls|sni=-|host=-|path=-
```

## Generation

The dedup key is generated from the `Node` struct:

```rust
impl Node {
    pub fn dedup_key(&self) -> NodeDedupKey {
        NodeDedupKey {
            protocol: self.protocol.clone(),
            address: self.address.clone(),
            port: self.port,
            username: self.username.clone(),
            uuid: self.uuid.clone(),
            password: self.password.clone(),
            method: self.method.clone(),
            network: self.network.clone(),
            tls: self.tls.clone(),
            sni: self.sni.clone(),
            host: self.host.clone(),
            path: self.path.clone(),
        }
    }
    
    pub fn dedup_key_string(&self) -> String {
        self.dedup_key().to_string()
    }
}
```

### Formatting

```rust
impl fmt::Display for NodeDedupKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("v1")?;
        write_required(f, "protocol", self.protocol.as_str())?;
        write_required(f, "address", &self.address)?;
        write_required(f, "port", &self.port.to_string())?;
        write_optional(f, "username", self.username.as_deref())?;
        write_optional(f, "uuid", self.uuid.as_deref())?;
        write_optional(f, "password", self.password.as_deref())?;
        write_optional(f, "method", self.method.as_deref())?;
        write_required(f, "network", &self.network)?;
        write_optional(f, "tls", self.tls.as_deref())?;
        write_optional(f, "sni", self.sni.as_deref())?;
        write_optional(f, "host", self.host.as_deref())?;
        write_optional(f, "path", self.path.as_deref())?;
        Ok(())
    }
}

fn write_required(f: &mut fmt::Formatter<'_>, name: &str, value: &str) -> fmt::Result {
    write!(f, "|{}={}:{}", name, value.chars().count(), value)
}

fn write_optional(f: &mut fmt::Formatter<'_>, name: &str, value: Option<&str>) -> fmt::Result {
    match value {
        Some(value) => write_required(f, name, value),
        None => write!(f, "|{}=-", name),
    }
}
```

## Database Storage

The dedup key is stored in the `configs` table:

```sql
CREATE TABLE configs (
    id INTEGER PRIMARY KEY,
    subscription_id INTEGER,
    dedup_key TEXT NOT NULL UNIQUE,
    protocol TEXT NOT NULL,
    address TEXT NOT NULL,
    port INTEGER NOT NULL,
    -- ... other fields
);
```

The `UNIQUE` constraint on `dedup_key` enforces uniqueness at the database level.

## Import Behavior

When importing configs:

```rust
pub async fn import_nodes(&self, nodes: Vec<Node>, subscription_id: i64) -> Result<ImportSummary> {
    let mut inserted = 0;
    let mut duplicates = 0;
    
    for node in nodes {
        let dedup_key = node.dedup_key_string();
        
        match self.insert_config(&node, subscription_id, &dedup_key).await {
            Ok(_) => inserted += 1,
            Err(DbError::UniqueViolation) => duplicates += 1,
            Err(e) => return Err(e),
        }
    }
    
    Ok(ImportSummary { inserted, duplicates })
}
```

### Unique Violation

If a config with the same dedup key already exists, the database returns a
unique violation error, which is caught and counted as a duplicate.

## Edge Cases

### Different Names, Same Config

Two configs with different display names but identical connection parameters
are considered duplicates:

```
vless://uuid@example.com:443?type=ws#Node1
vless://uuid@example.com:443?type=ws#Node2
```

Both generate the same dedup key (name is not included), so only one is
imported.

### None vs Empty String

These are treated as different:

```rust
let key1 = NodeDedupKey { password: None, .. };
let key2 = NodeDedupKey { password: Some("".to_string()), .. };

assert_ne!(key1.to_string(), key2.to_string());
```

Key 1: `|password=-`
Key 2: `|password=0:`

### Protocol Differences

Different protocols with the same address/port are not duplicates:

```
vless://uuid@example.com:443
vmess://uuid@example.com:443
```

Different `protocol` field → different dedup keys.

## Versioning

The `v1` prefix allows future format changes without breaking existing data:

- **v1**: Current format (length-prefixed)
- **v2**: Future format (if needed)

When reading dedup keys, xrat checks the version prefix and handles each
version appropriately.

## Testing

xrat includes tests to verify dedup behavior:

```rust
#[test]
fn distinguishes_none_from_empty_string() {
    let none_key = NodeDedupKey {
        protocol: Protocol::Ss,
        address: "example.com".to_string(),
        port: 8388,
        username: None,
        uuid: None,
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: None,
        sni: None,
        host: None,
        path: None,
    };
    let empty_key = NodeDedupKey {
        password: Some(String::new()),
        ..none_key.clone()
    };
    
    assert_ne!(none_key.to_string(), empty_key.to_string());
    assert!(empty_key.to_string().contains("|password=0:"));
}
```

## Performance

Dedup key generation is fast:

- String formatting: ~1-2 microseconds per key
- Database lookup: indexed on `dedup_key` column
- Import performance: ~1000 configs/second (including dedup)

## Related

- [Importing](importing.md) — how dedup is used during import
- [Database Schema](../05-reference/database-schema.md) — `configs` table
- [Protocols](../05-reference/protocols.md) — supported protocols
