# Config Import Module

This module handles importing proxy configurations from various sources and
formats.

## Supported Formats

### 1. Single Share Links

Direct proxy configuration URLs:

```txt
vless://uuid@example.com:443?type=ws&security=tls#MyNode
vmess://base64EncodedJson
ss://base64UserInfo@server:port#name
trojan://password@server:port?security=tls#name
http://proxy.example.com:8080
socks5://user:pass@proxy.example.com:1080
```

### 2. Base64-Encoded Subscriptions

A base64-encoded list of share links (one per line):

```txt
dmxlc3M6Ly91dWlkQGV4YW1wbGUuY29tOjQ0Mz90eXBlPXdzI05vZGUxCnZtZXNzOi8vYmFzZTY0SnNvbiNOb2RlMg==
```

When decoded, contains:

```txt
vless://uuid@example.com:443?type=ws#Node1
vmess://base64Json#Node2
```

### 3. Plain Text Lists

Multiple share links, one per line:

```txt
vless://uuid@example.com:443?type=ws#Node1
vmess://base64Json#Node2
ss://base64@server:port#Node3
```

Can include metadata:

```txt
STATUS=Active
vless://uuid@example.com:443#Node1
vmess://base64Json#Node2
```

### 4. SIP008 JSON (Shadowsocks)

Standard Shadowsocks subscription format:

```json
{
  "version": 1,
  "servers": [
    {
      "server": "example.com",
      "server_port": 8388,
      "method": "aes-256-gcm",
      "password": "secret",
      "remarks": "My Server",
      "plugin": "",
      "plugin_opts": ""
    }
  ]
}
```

### 5. Full Xray JSON Config

Complete Xray configuration (currently for validation only):

```json
{
  "log": {
    "loglevel": "warning"
  },
  "inbounds": [...],
  "outbounds": [...]
}
```

## Usage

### Basic Import

```rust
use xrat::config::{parse_import, ImportMode};

// Auto-detect format
let result = parse_import(input, ImportMode::Auto)?;

// Explicit format
let result = parse_import(input, ImportMode::Base64Subscription)?;

// Check results
println!("Parsed {} nodes", result.nodes.len());
for (line, error) in result.errors {
    eprintln!("Line {}: {}", line, error);
}
```

### Fetch Subscription URL

```rust
use xrat::config::fetch_subscription;

// Fetch and parse subscription
let result = fetch_subscription("https://example.com/sub").await?;

// Check metadata
if let Some(meta) = result.metadata {
    println!("Upload: {:?}", meta.upload);
    println!("Download: {:?}", meta.download);
    println!("Total: {:?}", meta.total);
    println!("Expire: {:?}", meta.expire);
}
```

### Special URL Formats

#### sub:// Protocol

Base64-encoded subscription URL:

```rust
// sub://aHR0cHM6Ly9leGFtcGxlLmNvbS9zdWI=
// Decodes to: https://example.com/sub
let result = fetch_subscription("sub://aHR0cHM6Ly9leGFtcGxlLmNvbS9zdWI=").await?;
```

#### Bare Domain

Automatically adds `http://` prefix:

```rust
// example.com/subscription
// Becomes: http://example.com/subscription
let result = fetch_subscription("example.com/subscription").await?;
```

## Import Modes

### Auto Detection

The parser automatically detects the format based on content:

- Starts with `{` → JSON (SIP008 or Xray)
- Contains `"inbounds"` or `"version"` → Xray JSON
- Contains `"servers"` → SIP008 JSON
- Single line with protocol → Single share link
- Multiple lines with protocol → Plain list
- Otherwise → Base64 subscription

### Explicit Modes

- `ImportMode::Auto` - Auto-detect (default)
- `ImportMode::SingleLink` - Single share link
- `ImportMode::Base64Subscription` - Base64-encoded list
- `ImportMode::PlainList` - Plain text list
- `ImportMode::Sip008Json` - SIP008 JSON
- `ImportMode::XrayJson` - Full Xray config

## Error Handling

### Best-Effort Parsing

For batch imports (lists, subscriptions), the parser uses best-effort:

- Invalid lines are skipped
- Errors are collected in `ImportResult.errors`
- Valid nodes are still returned

```rust
let result = parse_import(input, ImportMode::PlainList)?;

if !result.errors.is_empty() {
    eprintln!("Encountered {} errors:", result.errors.len());
    for (line, error) in result.errors {
        eprintln!("  Line {}: {}", line, error);
    }
}

println!("Successfully parsed {} nodes", result.nodes.len());
```

### Strict Parsing

For single links, parsing is strict:

```rust
match parse_import(input, ImportMode::SingleLink) {
    Ok(result) => println!("Parsed: {:?}", result.nodes[0]),
    Err(e) => eprintln!("Failed to parse: {}", e),
}
```

## Subscription Metadata

### From Headers

The `Subscription-Userinfo` header provides usage statistics:

```txt
Subscription-Userinfo: upload=1024; download=2048; total=10240; expire=1234567890
```

Parsed into:

```rust
SubscriptionMetadata {
    upload: Some(1024),      // bytes uploaded
    download: Some(2048),    // bytes downloaded
    total: Some(10240),      // total bandwidth
    expire: Some(1234567890), // expiration timestamp
    status: None,
}
```

### From Content

`STATUS=` lines in the content:

```txt
STATUS=Active
vless://...
vmess://...
```

Parsed into:

```rust
SubscriptionMetadata {
    upload: None,
    download: None,
    total: None,
    expire: None,
    status: Some("Active".to_string()),
}
```

## Supported Protocols

Current protocol support:

- ✅ VLESS (`vless://`)
- ✅ VMess (`vmess://`)
- ✅ Shadowsocks (`ss://`)
- ✅ Trojan (`trojan://`)
- ✅ HTTP/HTTPS (`http://`, `https://`)
- ✅ SOCKS5 (`socks5://`)
- ❌ ShadowsocksR (deprecated, not supported)

## Comparison with v2rayA

Based on v2rayA's implementation, this module provides:

### Similarities

- ✅ Single link parsing
- ✅ Base64 subscription decoding
- ✅ SIP008 JSON support
- ✅ Batch import with error tolerance
- ✅ Subscription URL fetching
- ✅ Subscription-Userinfo header parsing
- ✅ STATUS metadata extraction

### Improvements

- ✅ Explicit error reporting (not just logs)
- ✅ Structured `ImportResult` with errors list
- ✅ Auto-detection with explicit mode override
- ✅ No deprecated protocols (SSR removed)
- ✅ Type-safe parsing modes
- ✅ Comprehensive test coverage

### Not Yet Implemented

- ⏳ Full Xray JSON → Node extraction
- ⏳ Plugin protocol support
- ⏳ Custom protocol extensions

## Examples

### Example 1: Import from Clipboard

```rust
use xrat::config::parse_import;

let clipboard_content = get_clipboard_text();
let result = parse_import(&clipboard_content, ImportMode::Auto)?;

for node in result.nodes {
    println!("Found: {} ({}:{})",
        node.name.unwrap_or_default(),
        node.address,
        node.port
    );
}
```

### Example 2: Fetch and Update Subscription

```rust
use xrat::config::fetch_subscription;

async fn update_subscription(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let result = fetch_subscription(url).await?;

    println!("Fetched {} nodes", result.nodes.len());

    if let Some(meta) = result.metadata {
        if let Some(expire) = meta.expire {
            let days_left = (expire - now()) / 86400;
            println!("Subscription expires in {} days", days_left);
        }
    }

    // Save nodes to database
    for node in result.nodes {
        save_node(&node)?;
    }

    Ok(())
}
```

### Example 3: Validate Before Import

```rust
use xrat::config::parse_import;

fn import_with_validation(input: &str) -> Result<Vec<Node>, String> {
    let result = parse_import(input, ImportMode::Auto)
        .map_err(|e| format!("Parse error: {}", e))?;

    if result.nodes.is_empty() {
        return Err("No valid nodes found".to_string());
    }

    if !result.errors.is_empty() {
        eprintln!("Warning: {} lines failed to parse", result.errors.len());
    }

    Ok(result.nodes)
}
```

## Testing

Run tests:

```bash
cargo test config::import
```

Test coverage includes:

- Format auto-detection
- Single link parsing
- Base64 decoding
- Plain list parsing
- SIP008 JSON parsing
- Error handling
- Metadata extraction
- URL normalization
