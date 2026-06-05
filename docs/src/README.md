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

- **Import** subscriptions, files, raw links, base64 lists, SIP008 JSON, and Xray JSON into SQLite or PostgreSQL.
- **Support** VLESS, VMess, Trojan, Shadowsocks, HTTP, SOCKS5, and Hysteria2 parsing/preview.
- **Deduplicate** configs with normalized, versioned keys while preserving subscription metadata.
- **Test** proxies with ICMP, TCP, real-delay, download, and upload stages.
- **Rank** bulk test results with concurrency control, failure classification, history, and GeoIP enrichment.
- **Run** Xray-core or V2Ray-core as a managed local proxy runtime.
- **Expose** SOCKS5, HTTP, and Shadowsocks inbounds with configurable ports and sniffing.
- **Supervise** runtime sessions through a daemon with IPC, health checks, and stale-session reattach.
- **Rotate** proxies automatically on schedule or health failure, with cooldown and manual override.
- **Scan** Cloudflare/CDN edge IPs and persist reachable endpoints with latency.
- **Control** configs through CLI, interactive TUI, HTTP API, or systemd user services.
- **Serve** stored configs as JSON or base64 subscriptions with optional API-key authentication.
- **Manage** config state with enable, disable, soft delete, restore, purge, and detailed show commands.
- **Inspect** operational events with `xrat logs` for daemon, runtime, rotation, health, and test activity.
- **Generate** shell completions, man pages, Docker images, and self-upgrade from releases or source.

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
