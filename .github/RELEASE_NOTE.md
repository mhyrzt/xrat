## xrat v0.20.0

This release expands sing-box from Hysteria2-only managed sessions to every
protocol xrat imports, and adds native Xray Hysteria2 output.

### Features

- **Managed sing-box for imported protocols.** VLESS, VMess, Trojan,
  Shadowsocks, HTTP/HTTPS, SOCKS5, and Hysteria2 can generate sing-box runtime
  configs and run through `xrat connect` when `[runtime].engine = "sing-box"`.
  TLS, transports, inbounds, DNS, routing, and Clash API settings use sing-box
  1.13 shapes; unsupported settings return errors instead of being dropped.
- **Engine-aware testing.** `xrat test` uses sing-box probes when the runtime
  engine is sing-box. `xrat parse --engine sing-box` previews the
  generated JSON. Auto preview continues to select sing-box for Hysteria2 and
  Xray for other protocols.
- **Xray Hysteria2.** Xray can generate and run Hysteria2 outbounds when the
  imported URI fields can be represented by Xray.
- **Pinned managed installation.** `xrat setup` installs sing-box v1.13.21.
  User-supplied sing-box binaries must be at least v1.13.0; managed sessions
  check the generated config with the selected binary before launch.

### Upgrade notes

- No database migration or manual configuration change is required.
- sing-box GeoIP and geosite routing downloads remote SagerNet rule sets on
  first use; it needs network access and cannot reuse Xray `.dat` assets.
- Representative generated configs have passed native sing-box checks. The
  complete fixture matrix and pinned validator CI gate are still pending, so
  this release does not claim exhaustive conformance for sing-box 1.13 or 1.14.
  Newer sing-box versions are accepted but may reject deprecated generated
  fields during managed-session preflight.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.19.1...v0.20.0
