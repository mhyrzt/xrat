# Reference: xray-knife link parser field coverage

### Status

Reference (no work item; supports `config-roundtrip-fidelity.md`)

### What this is

[`xray-knife`](https://github.com/lilendian0x00/xray-knife) (Go, dual xray-core

- sing-box) solves the same link → runtime-config problem xrat does. This
  records exactly which parameters its xray parsers read, as a target for xrat's
  parser rework. Source: `pkg/core/xray/{vless,trojan,vmess}.go` on `master`,
  inspected 2026-06-20.

### How it differs from xrat

xray-knife reads every transport/security parameter explicitly into typed struct
fields, for **each** protocol — including Trojan, which xrat parses with
`extensions: None`. It also validates `sni`/`host`, applies sensible defaults,
and treats a missing `type` as `tcp` and a missing Trojan `security` as `tls`.

### VLESS (`pkg/core/xray/vless.go`)

Reads from the query string:

- `security` (`tls` / `reality` / none), `type` (network), `alpn`, `fp`
  (fingerprint), `sni`, `host`, `path`
- `flow`, `pbk` (reality public key), `sid` (reality short id), `spx` (spiderX)
- `headerType` (e.g. `http` for TCP obfuscation), `serviceName` (gRPC), `mode`
  (gRPC gun/multi or xhttp mode), `quicSecurity`

Defaults: `path` → `/` for `headerType=http`/`ws`/`h2`/`xhttp`; `type` → `tcp`
when empty.

### Trojan (`pkg/core/xray/trojan.go`)

Reads the same broad set: `flow`, `security`, `alpn`, `fp`, `type`, `sni`,
`host`, `path`, `headerType`, `serviceName`, `mode`, `pbk`, `sid`, `spx`,
`quicSecurity`.

Defaults: `type` → `tcp`; `security` → `tls` when empty; `fp` → `chrome` when
`security` is `tls`/`reality`; `flow` → `none` for gRPC.

### VMess (`pkg/core/xray/vmess.go`)

Base64 JSON or legacy query form. Reads `security`/`scy` (cipher) from userinfo,
`path`, `host`, `sni`, `alterId`, and maps legacy `obfs=websocket` → ws and
`obfsParam` → host. Normalizes `type`/network and defaults `path` → `/` for
http-family transports.

### Gap table vs xrat

| Parameter         | xray-knife (vless/trojan) | xrat today                         |
| ----------------- | ------------------------- | ---------------------------------- |
| `flow`            | both                      | vless only (added v0.9.0)          |
| `pbk`/`sid`/`spx` | both                      | vless only                         |
| `fp`/`alpn`       | both                      | vless only                         |
| `headerType`      | both                      | dropped                            |
| `serviceName`     | both                      | dropped (grpc uses `path` instead) |
| `mode`            | both                      | vless only                         |
| `quicSecurity`    | both                      | dropped                            |
| Trojan params     | full set                  | `extensions: None` (all dropped)   |
| VMess `scy`/`aid` | read                      | dropped                            |

### Takeaways for the rework

- Read parameters per protocol, not via a vless-only allowlist; Trojan and VMess
  need the same coverage (Trojan+REALITY is otherwise broken like the v0.9.0
  bug).
- Read `serviceName` for gRPC instead of overloading `path`.
- Adopt the same defaulting rules (`type`→`tcp`, Trojan `security`→`tls`,
  `fp`→`chrome` for tls/reality) so generated configs match common clients.
- Validate `sni`/`host` on parse, as xray-knife does, to fail early on garbage
  entries (the junk `info` / `2087` addresses we saw would be caught here).
