<p align="center">
  <img src="media/icons/xrat-icon-1024x1024.png" alt="xrat — proxy manager for XTLS/Xray-core and SagerNet/sing-box" width="256">
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

- **Import** subscriptions, files, raw links, base64 lists, SIP008 JSON, and
  Xray JSON into SQLite or PostgreSQL.
- **Support** VLESS, VMess, Trojan, Shadowsocks, HTTP, SOCKS5, and Hysteria2
  parsing/preview.
- **Deduplicate** configs with normalized, versioned keys while preserving
  subscription metadata.
- **Test** proxies with ICMP, TCP, real-delay, download, and upload stages.
- **Rank** bulk test results with concurrency control, failure classification,
  history, and GeoIP enrichment.
- **Run** Xray-core, V2Ray-core, or sing-box-backed Hysteria2 as a managed local
  proxy runtime.
- **Expose** SOCKS5, HTTP, and Shadowsocks inbounds with configurable ports and
  sniffing.
- **Supervise** runtime sessions through a daemon with IPC, health checks, and
  stale-session reattach.
- **Rotate** proxies automatically on schedule or health failure, with cooldown
  and manual override.
- **Scan** Cloudflare/CDN edge IPs and persist reachable endpoints with latency.
- **Control** configs through CLI, interactive TUI, HTTP API, or systemd user
  services.
- **Serve** stored configs as JSON or base64 subscriptions with optional API-key
  authentication.
- **Manage** config state with enable, disable, soft delete, restore, purge, and
  detailed show commands.
- **Inspect** operational events with `xrat logs` for daemon, runtime, rotation,
  health, and test activity.
- **Generate** shell completions, man pages, Docker images, and self-upgrade
  from releases or source.

[SagerNet/sing-box](https://github.com/sagernet/sing-box) support covers
sing-box JSON preview and managed Hysteria2 runtime sessions. Hy2 configs
automatically launch through sing-box because Xray/V2Ray cannot express that
protocol; other managed runtime protocols still use Xray/V2Ray unless
`[runtime].engine` selects a supported engine.

## Sections

| Section                                          | Description                                     |
| ------------------------------------------------ | ----------------------------------------------- |
| [Getting Started](01-getting-started/index.html) | Installation, quickstart, configuration         |
| [CLI Reference](02-cli/index.html)               | Command reference for all subcommands           |
| [Features](03-features/index.html)               | Deep-dives into each major subsystem            |
| [Deployment](04-deployment/index.html)           | systemd services, database backends             |
| [Reference](05-reference/index.html)             | Protocols, config file, database schema, errors |
| [Architecture](06-architecture/index.html)       | Module map, config generation pipeline          |
