# Protocols

xrat supports 7 proxy protocols, each with specific URI formats, configuration
fields, and engine routing.

## Supported Protocols

| Protocol    | URI Scheme                | Xray | sing-box | Parser |
| ----------- | ------------------------- | ---- | -------- | ------ |
| VLESS       | `vless://`                | Yes  | No       | Yes    |
| VMess       | `vmess://`                | Yes  | No       | Yes    |
| Shadowsocks | `ss://`                   | Yes  | No       | Yes    |
| Trojan      | `trojan://`               | Yes  | No       | Yes    |
| HTTP        | `http://` / `https://`    | Yes  | No       | Yes    |
| SOCKS5      | `socks5://`               | Yes  | No       | Yes    |
| Hysteria2   | `hysteria2://` / `hy2://` | No   | Yes      | Yes    |

---

## VLESS

Modern, lightweight protocol from the Xray project.

**Scheme**: `vless://`

**Format**:

```
vless://<uuid>@<address>:<port>?type=<network>&security=<tls>&sni=<sni>&host=<host>&path=<path>#<name>
```

**Fields**:

| Field      | Location | Required | Description                                                 |
| ---------- | -------- | -------- | ----------------------------------------------------------- |
| `uuid`     | userinfo | Yes      | VLESS user ID                                               |
| `address`  | host     | Yes      | Server address                                              |
| `port`     | port     | Yes      | Server port                                                 |
| `type`     | query    | No       | Network type (`tcp`, `ws`, `grpc`, `xhttp`), default `tcp`  |
| `security` | query    | No       | Security mode (`tls`, `reality`, `none`), default `none`    |
| `sni`      | query    | No       | SNI hostname                                                |
| `host`     | query    | No       | Host header (WebSocket)                                     |
| `path`     | query    | No       | Path (WebSocket, gRPC, TCP)                                 |
| `flow`     | query    | No       | Flow control, e.g. `xtls-rprx-vision`                       |
| `fp`       | query    | No       | uTLS fingerprint, e.g. `chrome` (REALITY defaults `chrome`) |
| `alpn`     | query    | No       | Comma-separated ALPN list (TLS)                             |
| `mode`     | query    | No       | xhttp/gRPC mode, e.g. `packet-up`                           |
| `pbk`      | query    | REALITY  | REALITY public key (required when `security=reality`)       |
| `sid`      | query    | No       | REALITY short ID                                            |
| `spx`      | query    | No       | REALITY spiderX path                                        |
| `name`     | fragment | No       | Display name                                                |

**REALITY**: when `security=reality`, xrat builds current Xray `realitySettings`
from `pbk`/`password`, `sid`, `spx`, `fp`, and `sni`. The share-link public key
is emitted as Xray's current `password` field and is required. REALITY is
accepted only with raw, xhttp, or gRPC transports.

All non-structural query parameters are preserved. xrat generates current raw,
WebSocket, gRPC, xhttp, mKCP, and HTTPUpgrade settings when representable and
fails with a named unsupported-parameter/transport error instead of silently
dropping a wire-affecting value.

For xhttp, current flat parameters use Xray's camelCase names, including
`xPaddingBytes`, `xPaddingObfsMode`, `noGRPCHeader`, `noSSEHeader`, `headers`,
`xmux`, and `downloadSettings`. The compatibility aliases `x_padding_bytes` and
the URL-encoded `x_padding%20bytes` are also accepted for `xPaddingBytes`.

Newer Xray xhttp options can be passed in the URL-encoded `extra` query
parameter as a JSON object. Fields inside `extra` are preserved so links can use
options introduced by newer Xray versions. A canonical flat parameter overrides
the same field in `extra`; `extra` overrides compatibility aliases. Unknown flat
parameters, malformed values, repeated singular parameters, and conflicting
aliases are rejected. Whether a future field works at runtime still depends on
the installed Xray version.

**Examples**:

```
vless://uuid-123@example.com:443?type=ws&security=tls&sni=cdn.example.com&path=%2Fray#My%20Node
vless://uuid-456@example.com:443?type=tcp#Direct
vless://uuid-789@example.com:8443?type=grpc&serviceName=service#gRPC%20Node
vless://uuid-abc@example.com:8080?type=xhttp&security=reality&sni=www.example.com&pbk=PUBLICKEY&sid=SHORTID&fp=chrome&flow=xtls-rprx-vision#REALITY%20Node
vless://uuid-def@example.com:443?type=xhttp&mode=auto&xPaddingBytes=100-1000&extra=%7B%22noSSEHeader%22%3Atrue%7D#XHTTP%20Node
```

**Engine**: Xray (auto), Xray (explicit)

---

## VMess

Legacy protocol with encryption, from V2Ray.

**Scheme**: `vmess://`

**Format**:

```
vmess://<base64-json>
```

**Base64 JSON Fields**:

```json
{
  "add": "example.com",
  "port": "443",
  "id": "uuid-456",
  "net": "ws",
  "tls": "tls",
  "sni": "edge.example.com",
  "host": "host.example.com",
  "path": "/vmess",
  "ps": "VMess Node"
}
```

| Field  | Key  | Required | Description                               |
| ------ | ---- | -------- | ----------------------------------------- |
| `add`  | JSON | Yes      | Server address                            |
| `port` | JSON | Yes      | Server port                               |
| `id`   | JSON | No       | UUID                                      |
| `net`  | JSON | No       | Network type (`tcp`, `ws`), default `tcp` |
| `tls`  | JSON | No       | TLS mode (`tls`)                          |
| `sni`  | JSON | No       | SNI hostname                              |
| `host` | JSON | No       | Host header (WebSocket)                   |
| `path` | JSON | No       | Path (WebSocket)                          |
| `ps`   | JSON | No       | Display name                              |

**Example**:

```
vmess://eyJhZGQiOiJleGFtcGxlLmNvbSIsInBvcnQiOiI0NDMiLCJpZCI6InV1aWQtNDU2IiwibmV0Ijoid3MiLCJ0bHMiOiJ0bHMiLCJzbmkiOiJlZGdlLmV4YW1wbGUuY29tIiwiaG9zdCI6Imhvc3QuZXhhbXBsZS5jb20iLCJwYXRoIjoiL3ZtZXNzIiwicHMiOiJWTWVzcyBOb2RlIn0=
```

**Engine**: Xray (auto), Xray (explicit)

---

## Shadowsocks

Simple, secure proxy protocol.

**Scheme**: `ss://`

**Format**:

```
ss://<base64(method:password)>@<address>:<port>#<name>
```

**Fields**:

| Field      | Location        | Required | Description       |
| ---------- | --------------- | -------- | ----------------- |
| `method`   | base64 userinfo | Yes      | Encryption method |
| `password` | base64 userinfo | Yes      | Password          |
| `address`  | host            | Yes      | Server address    |
| `port`     | port            | Yes      | Server port       |
| `name`     | fragment        | No       | Display name      |

**Encryption methods**: `aes-128-gcm`, `aes-256-gcm`, `chacha20-ietf-poly1305`,
`xchacha20-ietf-poly1305`, `aes-128-cfb`, `aes-256-cfb`, `rc4-md5`

**Example**:

```
ss://YWVzLTI1Ni1nY206c2VjcmV0@example.com:8388#SS%20Node
```

**Engine**: Xray (auto), Xray (explicit)

---

## Trojan

TLS-based proxy that mimics HTTPS traffic.

**Scheme**: `trojan://`

**Format**:

```
trojan://<password>@<address>:<port>?type=<network>&sni=<sni>&host=<host>&path=<path>#<name>
```

**Fields**:

| Field      | Location | Required | Description                                       |
| ---------- | -------- | -------- | ------------------------------------------------- |
| `password` | userinfo | Yes      | Trojan password                                   |
| `address`  | host     | Yes      | Server address                                    |
| `port`     | port     | Yes      | Server port                                       |
| `type`     | query    | No       | Network type (`tcp`, `ws`, `grpc`), default `tcp` |
| `sni`      | query    | No       | SNI hostname                                      |
| `host`     | query    | No       | Host header (WebSocket)                           |
| `path`     | query    | No       | Path (WebSocket, gRPC)                            |
| `name`     | fragment | No       | Display name                                      |

**Default TLS**: Trojan always uses TLS (`security=tls` is added automatically).

**Examples**:

```
trojan://password@example.com:443?type=ws&sni=cdn.example.com&path=%2Ftrojan#Trojan%20Node
```

**Engine**: Xray (auto), Xray (explicit)

---

## HTTP

Standard HTTP/HTTPS proxy.

**Scheme**: `http://` / `https://`

**Format**:

```
http://<username>:<password>@<address>:<port>#<name>
https://<username>:<password>@<address>:<port>#<name>
```

**Fields**:

| Field      | Location | Required | Description    |
| ---------- | -------- | -------- | -------------- |
| `username` | userinfo | No       | Username       |
| `password` | userinfo | No       | Password       |
| `address`  | host     | Yes      | Server address |
| `port`     | port     | Yes      | Server port    |
| `name`     | fragment | No       | Display name   |

**TLS**: `https://` scheme automatically sets `tls=tls`.

**Examples**:

```
http://user:pass@example.com:8080#HTTP%20Node
https://example.com:443#HTTPS%20Node
```

**Engine**: Xray (auto), Xray (explicit)

---

## SOCKS5

Standard SOCKS5 proxy.

**Scheme**: `socks5://`

**Format**:

```
socks5://<username>:<password>@<address>:<port>#<name>
```

**Fields**:

| Field      | Location | Required | Description    |
| ---------- | -------- | -------- | -------------- |
| `username` | userinfo | No       | Username       |
| `password` | userinfo | No       | Password       |
| `address`  | host     | Yes      | Server address |
| `port`     | port     | Yes      | Server port    |
| `name`     | fragment | No       | Display name   |

**Examples**:

```
socks5://user:pass@example.com:1080#SOCKS%20Node
socks5://example.com:1080#Anonymous
```

**Engine**: Xray (auto), Xray (explicit)

---

## Hysteria2

QUIC-based protocol designed for high-speed connections.

**Scheme**: `hysteria2://` / `hy2://`

**Format**:

```
hysteria2://<password>@<address>:<port>?sni=<sni>&obfs=<type>&obfs-password=<pass>#<name>
hy2://<password>@<address>:<port>?sni=<sni>&obfs=<type>&obfs-password=<pass>#<name>
```

**Fields**:

| Field           | Location | Required | Description             |
| --------------- | -------- | -------- | ----------------------- |
| `password`      | userinfo | Yes      | Authentication password |
| `address`       | host     | Yes      | Server address          |
| `port`          | port     | Yes      | Server port             |
| `sni`           | query    | No       | SNI hostname            |
| `obfs`          | query    | No       | Obfuscation type        |
| `obfs-password` | query    | No       | Obfuscation password    |
| `alpn`          | query    | No       | ALPN protocol           |
| `insecure`      | query    | No       | Allow insecure TLS      |
| `upmbps`        | query    | No       | Upload Mbps             |
| `downmbps`      | query    | No       | Download Mbps           |
| `name`          | fragment | No       | Display name            |

**Default network**: `udp` (not configurable) **Default TLS**: `tls` (always
enabled)

**Examples**:

```
hy2://password@example.com:443?sni=cdn.example.com&obfs=salamander&obfs-password=secret#HY2%20Node
hy2://password@example.com:8443#Simple%20HY2
hysteria2://password@example.com:443#Alias
```

**Engine**: sing-box (auto), sing-box (explicit)

---

## Engine Routing

Engine selection is automatic but configurable.

### Auto Mode (Default)

| Protocol    | Engine   |
| ----------- | -------- |
| VLESS       | Xray     |
| VMess       | Xray     |
| Shadowsocks | Xray     |
| Trojan      | Xray     |
| HTTP        | Xray     |
| SOCKS5      | Xray     |
| Hysteria2   | sing-box |

### Xray Mode

All protocols except Hysteria2. Errors on Hysteria2.

### sing-box Mode

All protocols use sing-box (currently only Hysteria2 fully implemented).

### Checking Engine

```bash
xrat parse --engine auto "vless://uuid@example.com:443"
xrat parse --engine sing-box "hy2://password@example.com:443"
```

## Normalized Fields

All protocols are normalized to a common `Node` structure:

| Field    | VLESS    | VMess | SS       | Trojan   | HTTP        | SOCKS5   | HY2      |
| -------- | -------- | ----- | -------- | -------- | ----------- | -------- | -------- |
| protocol | vless    | vmess | ss       | trojan   | http        | socks5   | hy2      |
| address  | host     | add   | host     | host     | host        | host     | host     |
| port     | port     | port  | port     | port     | port/80/443 | port     | port     |
| uuid     | userinfo | id    | -        | -        | -           | -        | -        |
| password | -        | -     | base64   | userinfo | userinfo    | userinfo | userinfo |
| method   | -        | -     | base64   | -        | -           | -        | -        |
| network  | type     | net   | tcp      | type     | tcp         | tcp      | udp      |
| tls      | security | tls   | -        | tls      | scheme      | -        | tls      |
| sni      | sni      | sni   | -        | sni      | -           | -        | sni      |
| host     | host     | host  | -        | host     | -           | -        | -        |
| path     | path     | path  | -        | path     | -           | -        | -        |
| name     | fragment | ps    | fragment | fragment | fragment    | fragment | fragment |
