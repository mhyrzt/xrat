<p align="center">
  <img src="media/xrat-icon.png" alt="xrat — proxy manager for XTLS/Xray-core and SagerNet/sing-box" width="256">
</p>

# xrat 🐀

**xrat** is a command-line proxy configuration manager for
[**XTLS/Xray-core**](https://github.com/xtls/xray-core) and
[**SagerNet/sing-box**](https://github.com/sagernet/sing-box). Import
subscriptions, test latency, scan edge IPs, rotate proxies, and run a managed
[XTLS/Xray-core](https://github.com/xtls/xray-core) or
[V2Fly/V2Ray-core](https://github.com/v2fly/v2ray-core) proxy runtime — all from
a single Rust binary.

<p align="center">
  <img src="media/screenshot.png" alt="XRAT terminal UI showing proxy testing progress, config details, logs, and runtime status">
</p>

## What you can do

- **Import** proxy configs from subscription URLs, base64 files, or raw
  VLESS/VMESS/Trojan/Shadowsocks links
- **Test** proxies with real-delay, TCP ping, ICMP, download/upload speed
  measurements
- **Scan** Cloudflare edge IPs and persist working endpoints
- **Run** [XTLS/Xray-core](https://github.com/xtls/xray-core) or
  [V2Fly/V2Ray-core](https://github.com/v2fly/v2ray-core) as a managed runtime
  with automatic proxy rotation
- **Control** everything through CLI, terminal UI (TUI), HTTP API, or background
  daemon

[SagerNet/sing-box](https://github.com/sagernet/sing-box) support currently
covers parsing and runtime-config preview, including Hysteria2 diagnostics
through `xrat parse --engine sing-box`. Managed runtime process lifecycle is
[XTLS/Xray-core](https://github.com/xtls/xray-core) and
[V2Fly/V2Ray-core](https://github.com/v2fly/v2ray-core)-focused.

## Sections

| Section                                         | Description                                     |
| ----------------------------------------------- | ----------------------------------------------- |
| [Getting Started](01-getting-started/README.md) | Installation, quickstart, configuration         |
| [CLI Reference](02-cli/README.md)               | Command reference for all subcommands           |
| [Features](03-features/README.md)               | Deep-dives into each major subsystem            |
| [Deployment](04-deployment/README.md)           | systemd services, database backends             |
| [Reference](05-reference/README.md)             | Protocols, config file, database schema, errors |
| [Architecture](06-architecture/README.md)       | Module map, config generation pipeline          |
