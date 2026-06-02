<p align="center">
  <img src="media/xrat-hero.png" alt="xrat — proxy manager for Xray-core and sing-box" width="800">
</p>

# xrat 🐀

**xrat** is a command-line proxy configuration manager for **Xray-core** and
**sing-box**. Import subscriptions, test latency, scan edge IPs, rotate proxies,
and run a managed proxy runtime — all from a single Rust binary.

## What you can do

- **Import** proxy configs from subscription URLs, base64 files, or raw
  VLESS/VMESS/Trojan/Shadowsocks links
- **Test** proxies with real-delay, TCP ping, ICMP, download/upload speed
  measurements
- **Scan** Cloudflare edge IPs and persist working endpoints
- **Run** Xray-core or sing-box as a managed runtime with automatic proxy
  rotation
- **Control** everything through CLI, terminal UI (TUI), HTTP API, or background
  daemon

## Sections

| Section                                         | Description                                     |
| ----------------------------------------------- | ----------------------------------------------- |
| [Getting Started](01-getting-started/README.md) | Installation, quickstart, configuration         |
| [CLI Reference](02-cli/README.md)               | Command reference for all subcommands           |
| [Features](03-features/README.md)               | Deep-dives into each major subsystem            |
| [Deployment](04-deployment/README.md)           | systemd services, database backends             |
| [Reference](05-reference/README.md)             | Protocols, config file, database schema, errors |
| [Architecture](06-architecture/README.md)       | Module map, config generation pipeline          |
