# import

Import a subscription URL, file, or raw text into the database.

```bash
xrat import <input>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `input` | Subscription source: a URL, local file path, or raw subscription text |

## Input Formats

xrat automatically detects the input format:

| Format | Detection |
|--------|-----------|
| **Subscription URL** | Starts with `http://` or `https://` |
| **Local file** | Path to an existing file on disk |
| **Single share link** | Single line starting with a supported protocol scheme |
| **Base64 subscription** | Multi-line or single-line base64-encoded text |
| **Plain link list** | Multiple lines, each a valid share link |
| **SIP008 JSON** | JSON with `"servers"` array |
| **Xray JSON** | JSON with `"version"` or `"inbounds"` fields |

## Examples

Import from a subscription URL:

```bash
xrat import https://example.com/sub.txt
```

Import from a local file:

```bash
xrat import ./nodes.txt
```

Import raw base64 subscription text:

```bash
xrat import "dmxlc3M6Ly91dWlkQGV4YW1wbGUuY29tOjQ0Mw=="
```

Import a single share link:

```bash
xrat import "vless://uuid@example.com:443?type=ws&security=tls#MyNode"
```

## Behavior

1. Reads input from the specified source
2. Detects format automatically
3. Parses and normalizes each node
4. Deduplicates against existing configs using a versioned key
5. Persists new configs to the database
6. Creates or updates the subscription source record
7. Prints an import summary

## Related

- [`add`](#add) — add a single config URI without subscription tracking
- [`list configs`](list.md#list-configs) — view imported configs

---

# add

Add a single config URI directly to the database.

```bash
xrat add <input>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `input` | Config URI: `vless://...`, `vmess://...`, `ss://...`, `trojan://...`, `hysteria2://...`, etc. |

## Examples

```bash
xrat add "vless://uuid-123@example.com:443?type=ws&security=tls&sni=cdn.example.com#Node"
```

```bash
xrat add "ss://YWVzLTI1Ni1nY206c2VjcmV0@example.com:8388#SS%20Node"
```

## Behavior

Unlike `import`, `add` does not create a subscription source record. It parses
the single URI, normalizes it, deduplicates, and persists directly.
