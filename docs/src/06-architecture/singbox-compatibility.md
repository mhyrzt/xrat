# sing-box Compatibility Contract

xrat supports managed sing-box configurations for any sing-box version
`>=1.13.0`. There is no upper bound: newer stable and prerelease binaries are
accepted so users are not blocked by a hard ceiling. The conformance target is
[v1.13.21](https://github.com/SagerNet/sing-box/tree/v1.13.21), and the range
planned for conformance fixtures is `>=1.13.0, <1.15.0`. The tagged
[top-level options](https://github.com/SagerNet/sing-box/blob/v1.13.21/option/options.go)
and the corresponding official configuration pages are the source of truth for
every generated JSON field.

The runtime inspects `sing-box version` before writing a config or starting a
session. It rejects pre-1.13, malformed, and unavailable binaries with the
configured path, detected version, supported range, and a remediation. A version
outside the planned conformance range (for example `>=1.15.0`, `2.x`, or a
prerelease) is accepted with a warning instead of being rejected. Preflight
still runs `sing-box check -c <config>` with the actual binary before launch, so
an incompatible newer binary fails safely rather than starting a broken session.
Managed setup installs v1.13.21. The full native fixture matrix and release CI
gate for that version are pending (TASK-102 and TASK-109).

## Generated-shape Matrix

| Generated area                                                           | v1.13.21 authority                                                                                                                                        | Contract status                                                                                                                                                                        | Owner                   |
| ------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------- |
| Top-level `log`, `inbounds`, `outbounds`, `dns`, `route`, `experimental` | [options source](https://github.com/SagerNet/sing-box/blob/v1.13.21/option/options.go)                                                                    | Version-gated                                                                                                                                                                          | TASK-98 / TASK-73       |
| Hysteria2 outbound                                                       | [Hysteria2 outbound](https://sing-box.sagernet.org/configuration/outbound/hysteria2/)                                                                     | Supported strictly for password, `tcp`/`udp`, SNI, insecure, ALPN, Salamander, and integer bandwidth; unknown or lossy input fails                                                     | TASK-101                |
| VLESS outbound                                                           | [VLESS outbound](https://sing-box.sagernet.org/configuration/outbound/vless/)                                                                             | Supported: UUID, `xtls-rprx-vision` flow over direct TLS/REALITY, `packet_encoding`, TLS, transports                                                                                   | TASK-99.3               |
| VMess outbound                                                           | [VMess outbound](https://sing-box.sagernet.org/configuration/outbound/vmess/)                                                                             | Supported: UUID, `security`, `alter_id`, `packet_encoding`, TLS, transports                                                                                                            | TASK-99.5               |
| Trojan outbound                                                          | [Trojan outbound](https://sing-box.sagernet.org/configuration/outbound/trojan/)                                                                           | Supported: password, TLS, transports                                                                                                                                                   | TASK-99.6               |
| HTTP outbound                                                            | [HTTP outbound](https://sing-box.sagernet.org/configuration/outbound/http/)                                                                               | Supported: credentials, HTTPS `tls` block only when the import requires it                                                                                                             | TASK-99.2               |
| SOCKS outbound                                                           | [SOCKS outbound](https://sing-box.sagernet.org/configuration/outbound/socks/)                                                                             | Supported: version 5, credentials                                                                                                                                                      | TASK-99.4               |
| Shadowsocks outbound                                                     | [Shadowsocks outbound](https://sing-box.sagernet.org/configuration/outbound/shadowsocks/)                                                                 | Supported for documented AEAD/2022 and legacy methods; SIP003 plugins are rejected                                                                                                     | TASK-99.7               |
| TLS and V2Ray transports                                                 | [TLS](https://sing-box.sagernet.org/configuration/shared/tls/) and [V2Ray transport](https://sing-box.sagernet.org/configuration/shared/v2ray-transport/) | Supported: SNI, insecure, ALPN, uTLS fingerprint, REALITY, WebSocket, gRPC, HTTP, HTTPUpgrade, QUIC; unknown values fail                                                               | TASK-99.1               |
| SOCKS, HTTP, Shadowsocks inbounds                                        | [tagged inbound source](https://github.com/SagerNet/sing-box/blob/v1.13.21/option/inbound.go)                                                             | Supported as typed per-inbound output; SOCKS UDP cannot be disabled independently and HTTP has no auth settings in xrat                                                                | TASK-105                |
| Typed local, UDP, TCP, TLS, QUIC, HTTPS, HTTP/3, and hosts DNS servers   | [tagged DNS source](https://github.com/SagerNet/sing-box/blob/v1.13.21/option/dns.go)                                                                     | Version-gated; unsupported settings fail before launch and a route-level `default_domain_resolver` is emitted whenever DNS is present                                                  | TASK-103                |
| Domain and IP/CIDR route rules                                           | [route rules](https://sing-box.sagernet.org/configuration/route/rule/)                                                                                    | Version-gated for exact/suffix/keyword/regex domains and IP/CIDR                                                                                                                       | TASK-100.2              |
| GeoIP/geosite rules and rule-set assets                                  | [rule sets](https://sing-box.sagernet.org/configuration/rule-set/)                                                                                        | Supported via `remote` SagerNet `sing-geosite`/`sing-geoip` `.srs` rule-sets, cached with `experimental.cache_file`; Xray `.dat` assets are not a sing-box format and cannot be reused | TASK-100.1 / TASK-100.2 |
| `experimental.clash_api`                                                 | [Clash API](https://sing-box.sagernet.org/configuration/experimental/clash-api/)                                                                          | Version-gated; requires a loopback controller host and a port distinct from local inbounds                                                                                             | TASK-106                |

All generated configurations must pass `sing-box check -c <config>`, the same
command used by managed-runtime preflight. A missing validator is a skipped
local check, never a passing conformance result. The complete fixture matrix and
release enforcement belong to TASK-102 and TASK-109.

## Upstream Deprecation Watch

Because newer versions are accepted, track upstream deprecations that will
become removals. Verified against the official migration guide and a real
sing-box `1.14.1` binary:

| Emitted shape                                        | Upstream status                     | Action                                                                                |
| ---------------------------------------------------- | ----------------------------------- | ------------------------------------------------------------------------------------- |
| Remote rule-sets relying on the implicit HTTP client | Deprecated in 1.14, removed in 1.16 | Add `http_clients`/`default_http_client` for versions `>=1.14` before 1.16 (TASK-117) |
| Legacy DNS address-filter fields (`ip_accept_any`)   | Deprecated in 1.14                  | Migrate hosts routing to response matching if it is removed                           |
| `block` outbound with `outbound: "block"`            | Deprecated since 1.11               | Switch to `action: reject` when a bounded minimum rises past the removal              |
| Inline ACME, rule-set `download_detour`, TUN `stack` | Deprecated/removed 1.14–1.17        | Not emitted                                                                           |

Representative generated shapes have passed `sing-box check` on 1.13.x and
1.14.1; the full matrix remains pending.

## Implementation Rules

- Read the v1.13.21 documentation and tagged upstream option type before adding
  or changing each field.
- Generate a field only when xrat's normalized model preserves its complete
  sing-box meaning.
- Reject unrepresentable, deprecated, or unknown input with a field-specific
  error before process launch.
- Add a native validation fixture for every accepted JSON shape; run it against
  the pinned v1.13.21 binary.
- Update this matrix when the supported range or an emitted shape changes.
