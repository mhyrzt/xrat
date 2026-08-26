# Import Pipeline

The import pipeline converts raw input (URLs, files, raw text) into normalized
`Node` records, deduplicates them, and persists them as `ConfigRecord` rows
under their source `SubscriptionRecord`.

## End-to-End Flow

```mermaid
flowchart TD
    classDef io   fill:#1a2e1a,stroke:#5bdf8a,color:#e6edf3
    classDef cfg  fill:#2e2a1a,stroke:#dfba5b,color:#e6edf3
    classDef node fill:#1a2c3a,stroke:#5b8def,color:#e6edf3
    classDef db   fill:#1a2e2e,stroke:#5bcfdf,color:#e6edf3

    INPUT["Raw Input<br/>(URL / file / stdin)"]:::io
    READ["read_input()<br/>app/input/source.rs"]:::io
    SRC["ImportSource<br/>{ kind, value, name }"]:::io
    BYTES["raw bytes"]:::io
    IMPORT["run_import()<br/>app/import.rs"]:::cfg
    DETECT["detect format<br/>config/import/detect.rs"]:::cfg
    MODE["ImportMode<br/>(Auto | SingleLink | Base64 | Plain | Sip008 | XrayJson)"]:::cfg
    PARSE["parse_import()<br/>config/import/"]:::cfg
    RESULT["ImportResult<br/>{ nodes, errors, metadata }"]:::cfg
    NODE["model::Node"]:::node
    NORM["normalize()<br/>config/normalize.rs"]:::node
    DEDUP["dedup_key()<br/>model::Node"]:::node
    PERSIST["upsert<br/>db/repository/configs/import_ops"]:::db
    SUB_REC["link subscription<br/>db/repository/subscriptions/"]:::db

    INPUT --> READ
    READ --> SRC & BYTES
    SRC --> IMPORT
    BYTES --> IMPORT
    IMPORT --> DETECT --> MODE --> PARSE --> RESULT --> NODE --> NORM --> DEDUP --> PERSIST --> SUB_REC
```

## Input Sources

`app/input/source.rs::read_input` accepts a single string and decides how to
load the bytes. There is no separate `InputSource` enum at the call site — the
function returns an `ImportSource` plus the raw bytes.

```rust
pub fn read_input(input: &str) -> Result<(ImportSource, Vec<u8>), AppError>;

pub struct ImportSource {
    pub kind: SourceKind,    // Url | File | RawText
    pub value: String,       // original input or path
    pub name: Option<String>,
}
```

Decision rules:

- `looks_like_url(input)` → fetch with `reqwest::blocking::get`; status errors
  propagate as `AppError`.
- `Path::new(input).exists()` → read from disk; filename is used as `name` when
  available.
- otherwise → treat the input as raw text bytes.

## Format Detection

`config/import/detect.rs` runs heuristics on the (decoded) bytes to pick an
`ImportMode`:

```rust
pub enum ImportMode {
    Auto,
    SingleLink,
    Base64Subscription,
    PlainList,
    Sip008Json,
    XrayJson,
}
```

| Detected as          | Heuristic                                              | Parser entry                         |
| -------------------- | ------------------------------------------------------ | ------------------------------------ |
| `SingleLink`         | Starts with a known URI scheme                         | `parsers::parse_single_link`         |
| `Sip008Json`         | JSON object with `version: 1` and `servers: [...]`     | `parsers::parse_sip008_json`         |
| `XrayJson`           | JSON object with `outbounds` (or log/inbounds)         | `parsers::parse_xray_json`           |
| `Base64Subscription` | Decodes to base64, then plain-list parses successfully | `parsers::parse_base64_subscription` |
| `PlainList`          | Newline-separated share links                          | `parsers::parse_plain_list`          |

`config/import::parse_import` is the public entry point; it takes a string plus
an `ImportMode` (use `Auto` for detection, anything else to force a mode) and
returns `ImportResult { nodes, errors, metadata }`.

## Protocol Parsers

Each supported protocol has a dedicated parser in `config/protocols/`:

```mermaid
flowchart TD
    classDef input  fill:#1a2e1a,stroke:#5bdf8a,color:#e6edf3
    classDef parser fill:#2e2a1a,stroke:#dfba5b,color:#e6edf3
    classDef node   fill:#1a2c3a,stroke:#5b8def,color:#e6edf3
    classDef err    fill:#3a1a1a,stroke:#df5b5b,color:#e6edf3

    LINK["share link string"]:::input
    ROUTE{"URI scheme"}

    VL["vless.rs"]:::parser
    VM["vmess.rs"]:::parser
    SS["ss.rs"]:::parser
    TR["trojan.rs"]:::parser
    HT["http.rs"]:::parser
    SK["socks5.rs"]:::parser
    HY["hy2.rs"]:::parser
    ERR["ImportParseError<br/>::UnsupportedProtocol"]:::err
    NODE["model::Node"]:::node

    LINK --> ROUTE
    ROUTE -- "vless://" --> VL
    ROUTE -- "vmess://" --> VM
    ROUTE -- "ss://"    --> SS
    ROUTE -- "trojan://" --> TR
    ROUTE -- "http://"  --> HT
    ROUTE -- "socks5://" --> SK
    ROUTE -- "hy2://"   --> HY
    ROUTE -- "unknown"  --> ERR

    VL & VM & SS & TR & HT & SK & HY --> NODE
```

The `Protocol` enum lives in `model/protocol.rs` and serializes lowercase
(`vless`, `vmess`, `ss`, `trojan`, `http`, `socks5`, `hy2`).

## Node Struct

`model::Node` is the shared, in-memory domain type produced by every parser and
consumed by every engine generator. Fields are intentionally minimal —
structural fields stay typed while non-structural link parameters are preserved
as JSON-valued extensions and persisted explicitly.

```rust
pub struct Node {
    pub protocol: Protocol,
    pub address: String,
    pub port: u16,
    pub username: Option<String>,
    pub uuid: Option<String>,
    pub password: Option<String>,
    pub method: Option<String>,   // shadowsocks cipher
    pub network: String,         // tcp | ws | grpc | ...
    pub tls: Option<String>,
    pub sni: Option<String>,
    pub host: Option<String>,
    pub path: Option<String>,
    pub name: Option<String>,
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
    pub raw_config: String,
}
```

Node is also the source for `NodeDedupKey` (see below).

URL-form VLESS and VMess parsers reject repeated query parameters because the
official share-link format defines each field as singular. Legacy VMess JSON
preserves native booleans, numbers, arrays, and objects. The v2 dedup key includes
this deterministic extension map, so configurations that differ in a
wire-affecting parameter do not collapse into one row. Legacy rows are
backfilled from `raw_config` during schema initialization.

## Normalization

`config/normalize.rs` is a single `fn normalize(node: &mut Node)` that runs in
place after parsing:

- empty `network` becomes `"tcp"`
- `network == "ws"` → copy `sni` to `host` if `host` is `None`; set `path` to
  `"/"` if `path` is `None`
- `network == "grpc"` → set `path` to `"/"` if `path` is `None`
- empty-string `tls` (`Some("")`) is collapsed to `None`

## Deduplication

`model::Node::dedup_key` returns a `NodeDedupKey`. Its `Display` impl serializes
a length-prefixed, versioned string that the DB stores in `configs.dedup_key`
(UNIQUE).

```rust
pub struct NodeDedupKey {
    pub protocol: Protocol,
    pub address: String,
    pub port: u16,
    pub username: Option<String>,
    pub uuid: Option<String>,
    pub password: Option<String>,
    pub method: Option<String>,
    pub network: String,
    pub tls: Option<String>,
    pub sni: Option<String>,
    pub host: Option<String>,
    pub path: Option<String>,
}
```

The serialized form prefixes every field with `name=<char_count>:<value>` and
writes `name=-` for `None` values, so missing and empty-string values differ.
Example:

```text
v1|protocol=5:vless|address=11:example.com|port=3:443|username=-|uuid=8:uuid|123|password=-|method=-|network=2:ws|tls=3:tls|sni=15:cdn.example.com|host=15:cdn.example.com|path=4:/ray
```

`db/repository/configs/import_ops` performs the upsert keyed on `dedup_key`;
duplicates are skipped and counted in `ImportSummary`.
