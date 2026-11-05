# Importing

xrat imports proxy configurations from multiple sources and formats,
automatically detecting the input type and normalizing all configs into a
unified internal representation.

## Input Sources

### Subscription URL

Fetch configs from a remote HTTP endpoint:

```bash
xrat import https://example.com/subscription
```

xrat:

1. Fetches the URL content
2. Parses `subscription-userinfo` headers for metadata (upload, download, total,
   expire)
3. Detects format (base64, plain list, JSON)
4. Parses and normalizes each node
5. Persists to database with subscription tracking

### Local File

Import from a file on disk:

```bash
xrat import ./nodes.txt
```

Supports the same format detection as URLs.

### Raw Text

Import inline subscription text:

```bash
xrat import "vless://uuid@example.com:443?type=ws#Node"
```

Useful for quick imports or scripting.

## Input Formats

### Single Share Link

A single proxy URI:

```
vless://uuid-123@example.com:443?type=ws&security=tls&sni=cdn.example.com&path=%2Fray#My%20Node
```

Supported schemes:

- `vless://`
- `vmess://`
- `ss://`
- `trojan://`
- `http://` / `https://`
- `socks5://`
- `hysteria2://` / `hy2://`

### Base64 Subscription

Standard v2rayN/Clash subscription format:

```
dmxlc3M6Ly91dWlkQGV4YW1wbGUuY29tOjQ0Mz90eXBlPXdzJnNlY3VyaXR5PXRscyNNeSBOb2RlCnZtZXNzOi8v...
```

xrat:

1. Base64-decodes the payload
2. Splits into lines
3. Parses each line as a share link

### Plain Link List

Multiple share links, one per line:

```
vless://uuid-1@example.com:443?type=tcp#Node1
vmess://eyJhZGQiOiJleGFtcGxlLmNvbSIsInBvcnQiOiI0NDMifQ==#Node2
ss://YWVzLTI1Ni1nY206c2VjcmV0@example.com:8388#Node3
```

Lines starting with `#` are treated as comments and skipped.

### SIP008 JSON

The parser recognizes Shadowsocks SIP008 format, but JSON input is not currently
accepted by import:

```json
{
  "version": 1,
  "servers": [
    {
      "server": "example.com",
      "server_port": 8388,
      "method": "aes-256-gcm",
      "password": "secret",
      "remarks": "My SS Node"
    }
  ]
}
```

### Xray JSON

Import a complete Xray configuration as one native profile:

```bash
xrat import ./xray.json --name london-vpn
xrat connect <config-ref>
xrat show config <config-ref> --raw > restored.json
```

XRAT keeps a private copy of the original bytes, including custom XHTTP fields
such as `extra.xmux`, DNS, routing, multiple outbounds and unknown fields. It
does not normalize or inject settings into this JSON. Xray performs native
validation before an existing connection is stopped. Import checks JSON syntax
and shape; successful import alone does not prove the configuration is accepted
by Xray.

Profiles use `xray-json` as their catalog type and have no single proxy
endpoint. They support the existing references, names, enable/disable, soft
delete/restore, CLI/TUI/daemon connection lifecycle and individual raw export.
The TUI import dialog accepts a local file path or pasted JSON. A repeat import
of identical bytes reuses the reference and restores a soft-deleted profile;
changed bytes create a separate profile. Node probing, automatic rotation and
share-link subscriptions are unsupported. Export a profile individually instead
of joining its JSON with other records.

Runtime requirements:

- Select `runtime.engine = "xray"` and disable XRAT-managed TUN and kill switch.
  The daemon pauses automatic rotation while a raw profile is active and reports
  `rotation_paused_for_xray_profile`; reconnect a node and enable rotation to
  resume it. Raw TUN inbounds are rejected because their capture ownership
  cannot be verified.
- Include a local TCP SOCKS or HTTP inbound with a single nonzero port for
  startup readiness. Multiple inbounds are preserved; XRAT reports the first
  supported local endpoint of each type. It does not add an inbound when none is
  suitable.
- The profile owns its DNS, routing, TLS, authentication, logging and transport
  behavior. XRAT's generated-runtime settings do not override them. XRAT traffic
  statistics and automatic health probing are unavailable for raw profiles.
- Relative paths in the JSON retain Xray's process-working-directory semantics;
  use absolute paths for referenced certificates, keys and other files.

Local provisioning is required: browser uploads and machine-gateway imports
cannot introduce arbitrary engine configuration. Authorized browser users can
operate already provisioned profiles and retrieve their original JSON through
the existing protected raw-config endpoint. Profiles are limited to 4 MiB.

Back up both the database and the private `profiles/` directory under XRAT's
application root. Metadata contains a content digest and length; the JSON is not
stored in the database. Files are scoped to the database identity. A PostgreSQL
deployment must make the same private profile directory available to every
runtime owner using that database. Missing or changed files fail explicitly;
XRAT will not substitute generated node configuration. On Unix the profile
directories use mode `0700` and files use `0600`. Soft deletion retains the
file; permanent deletion through the application removes unreferenced profile
files. Disconnect a running profile before permanently deleting it.

If native validation fails, run `xray run -test -c ./xray.json` locally for full
diagnostics. XRAT keeps raw validation diagnostics out of general event messages
because they may contain credentials. Startup diagnostics are in the private
session error log.

## Format Detection

xrat automatically detects the input format using heuristics:

| Condition                                              | Detected Format                      |
| ------------------------------------------------------ | ------------------------------------ |
| JSON object with an `outbounds` array                  | Complete Xray profile (local import) |
| Starts with `{` and contains `"servers"`               | SIP008 JSON (unsupported for import) |
| Single line starting with a protocol scheme            | Single share link                    |
| Multiple lines, first line starts with protocol scheme | Plain link list                      |
| Otherwise                                              | Base64 subscription                  |

## Normalization

After parsing, xrat normalizes each node:

1. **Network defaults**: Empty network → `tcp`
2. **WebSocket defaults**: Missing `host` → copy from `sni`, missing `path` →
   `/`
3. **gRPC defaults**: Missing `path` → `/`
4. **TLS cleanup**: Empty string `tls` → `None`

## Deduplication

Before persisting, xrat generates a dedup key for each node and skips
duplicates. See [Deduplication](deduplication.md) for details.

## Subscription Tracking

Each import creates or updates a `subscriptions` record:

| Field               | Description                               |
| ------------------- | ----------------------------------------- |
| `source_url`        | Original URL or file path                 |
| `source_kind`       | `url`, `file`, or `raw_text`              |
| `name`              | Optional name (from URL or user-provided) |
| `created_at`        | First import timestamp                    |
| `updated_at`        | Latest import timestamp                   |
| `last_refreshed_at` | Last successful URL refresh (epoch secs)  |

Configs are linked to their subscription via `subscription_id` foreign key.

## Refreshing Subscriptions

Re-importing a URL-backed subscription is a reconciliation, not an additive
import: configs the provider still returns are upserted, and configs that
disappeared from a non-empty payload are automatically purged, including older
obsolete rows for that subscription and their test/runtime history. Configs
still active or referenced by a starting, running, or stopping runtime session
are retained as deleted until a later refresh can safely purge them. Other
subscriptions are unaffected. Import and cleanup run in one transaction; a
failed config import or empty payload removes nothing. A provider re-add creates
a new config row after the previous row has been purged. See
[Deduplication](deduplication.md) for the dedup key used to match configs.

There are two ways to refresh:

- **Manual** — run `xrat update` (or `xrat update <ref...>`), re-run
  `xrat import <url>`, or press `r` / `R` on the TUI Subscriptions tab.
  Available any time, no daemon required.
- **Automatic** — the daemon periodically re-fetches URL-backed subscriptions on
  a fixed interval. Configure it under `[subscriptions]`:

  ```toml
  [subscriptions]
  auto_refresh = false
  refresh_interval_hours = 24
  ```

  When `auto_refresh` is enabled, the daemon refreshes each URL-backed
  subscription whose `last_refreshed_at` is older than `refresh_interval_hours`
  (or that was never refreshed). Because the due check reads the persisted
  `last_refreshed_at`, intervals survive daemon restarts. Non-URL sources
  (files, raw text) are skipped, and a failed fetch is recorded as an event
  without stopping the daemon or the rest of the batch. Refresh start, success,
  and failure are visible in `xrat logs`.

  Automatic refresh requires a running daemon (`xrat daemon install --start`).
  Manual refresh uses the exact same import + reconciliation path.

## Metadata Extraction

For subscription URLs, xrat extracts metadata from HTTP headers:

```
subscription-userinfo: upload=1024; download=2048; total=10240; expire=1234567890
```

Parsed fields:

- `upload` — bytes uploaded
- `download` — bytes downloaded
- `total` — total quota
- `expire` — expiration timestamp (Unix epoch)

## Error Handling

xrat continues parsing even when individual lines fail:

```
Import Summary
─────────────────────────────────
Source:     https://example.com/sub.txt
Parsed:     45 nodes
Failed:     3 lines
Duplicates: 12 skipped
New:        33 configs added
```

Failed lines are logged with line numbers and error messages.

## Related

- [`import` CLI](../02-cli/import.md) — command reference
- [Deduplication](deduplication.md) — how duplicates are detected
- [Protocols](../05-reference/protocols.md) — supported protocol formats

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
