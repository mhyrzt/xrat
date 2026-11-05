# import

Import a subscription URL, file, or raw text into the database.

```bash
xrat import <input> [--name <name>] [--header 'Name: value']...
```

## Arguments

| Argument | Description                                                           |
| -------- | --------------------------------------------------------------------- |
| `input`  | Subscription source: a URL, local file path, or raw subscription text |

## Options

| Option                  | Description                                    |
| ----------------------- | ---------------------------------------------- |
| `-n`, `--name <name>`   | Name for the imported subscription source      |
| `--header <NAME:VALUE>` | HTTP header for URL subscriptions (repeatable) |

## Input Formats

xrat automatically detects the input format:

| Format                  | Detection                                             |
| ----------------------- | ----------------------------------------------------- |
| **Subscription URL**    | Starts with `http://` or `https://`                   |
| **Local file**          | Path to an existing file on disk                      |
| **Single share link**   | Single line starting with a supported protocol scheme |
| **Base64 subscription** | Multi-line or single-line base64-encoded text         |
| **Plain link list**     | Multiple lines, each a valid share link               |

Complete Xray JSON is imported from a local file or pasted JSON object with an
`outbounds` array as one profile, preserving the original file rather than
extracting individual nodes. SIP008 JSON subscriptions remain unsupported.
URL-backed JSON profiles and automatic refresh are unsupported; download the
file and import it locally. Browser uploads and machine-gateway imports cannot
provision arbitrary engine configuration; use the local CLI/TUI.

```bash
xrat import ./xray-conn-london.json --name london-vpn
xrat list
xrat connect <config-ref>
xrat show config <config-ref> --raw > restored.json
```

The `--raw` output contains credentials and is byte-for-byte identical to the
imported JSON. The original input file can be moved or deleted after import. See
[Xray JSON profiles](../03-features/importing.md#xray-json) for runtime and
backup requirements.

## Examples

Import from a subscription URL:

```bash
xrat import https://example.com/sub.txt
```

Import and name a subscription:

```bash
xrat import https://example.com/sub.txt --name "Work VPN"
```

Import from a local file:

```bash
xrat import ./nodes.txt
```

Import with custom headers:

```bash
xrat import https://example.com/sub.txt \
  --header 'X-HWID: device-id' \
  --header 'User-Agent: Happ'
```

Headers are saved for `update` and automatic refreshes. Each successful `import`
replaces the saved set; omitting `--header` clears it. With headers, redirects
must keep the same scheme, host, and port. Header values are stored unencrypted.

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
6. Creates or updates the subscription source record and applies `--name` when
   provided
7. Reconciles the source: configs attached to the same subscription that are
   absent from this payload are soft-deleted (recoverable). A later import that
   brings them back restores them. An empty payload removes nothing.
8. Prints an import summary, including the count of removed configs

For a new URL subscription without `--name`, the name comes from `Profile-Title`
(plain text or `base64:`), then the `Content-Disposition` filename, then the URL
fragment, then a `name`, `title`, `tag`, or `remark` query parameter. An
explicit name always wins, including names beginning with `sub-`.

Reimporting or refreshing a saved subscription without an explicit name
preserves its existing name. The TUI and Web UI follow the same rules. In the
TUI, accepting the suggested name leaves automatic detection enabled; typing a
name makes it explicit. The generated TUI suggestion is used when no metadata
supplies a name.

## Related

- [`add`](config-management.md#add) — add a single config URI without
  subscription tracking
- [`list configs`](list.md#list-configs) — view imported configs

---

# add

Add a single config URI directly to the database.

```bash
xrat add <input>
```

## Arguments

| Argument | Description                                                                                   |
| -------- | --------------------------------------------------------------------------------------------- |
| `input`  | Config URI: `vless://...`, `vmess://...`, `ss://...`, `trojan://...`, `hysteria2://...`, etc. |

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

For the full config-management command set, see
[`config management`](config-management.md).

## Subscription URL lists

Normal imports decode and parse subscription contents as proxy nodes. HTTP(S)
links inside a response or local node file are never fetched as subscriptions.
To import a list of subscription URLs, put one URL per line in a local file:

```bash
xrat import ./subscriptions.txt --url-list --name "Combined subscriptions"
```

Blank lines and lines starting with `#` are ignored. Every remaining line must
be an HTTP(S) URL. Each URL is fetched once; its response is parsed as nodes
without following embedded URLs. The results are merged under the local file
source, which is not automatically refreshed. Re-run the command to refresh it.
`--url-list` requires a local file and cannot be combined with `--header`.

Previously, imports automatically fetched HTTP(S) lines inside their contents.
Use `--url-list` for files that intentionally contain subscription URLs. For a
single HTTP(S) proxy URI supplied directly, use `xrat add "https://host:443"`.
