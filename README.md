<p align="center">
  <img src="docs/src/media/icons/xrat-icon-1024x1024.png" alt="xrat" width="256">
</p>

<h1 align="center">XRAT</h1>

<p align="center">
  <em>A fast, polished CLI and TUI proxy manager for <a href="https://github.com/xtls/xray-core">XTLS/Xray-core</a> and <a href="https://github.com/sagernet/sing-box">SagerNet/sing-box</a></em>
</p>

<p align="center">
  <img alt="Status" src="https://img.shields.io/badge/status-under%20development-orange">
  <img alt="Rust" src="https://img.shields.io/badge/rust-stable-blue">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green">
  <a href="https://mhyrzt.github.io/xrat"><img src="https://img.shields.io/badge/docs-mhyrzt.github.io%2Fxrat-blue" alt="Documentation"></a>
  <a href="https://crates.io/crates/xrat"><img src="https://img.shields.io/crates/v/xrat" alt="crates.io"></a>
</p>

XRAT is a Rust CLI and TUI for importing, storing, testing, and running
[XTLS/Xray-core](https://github.com/xtls/xray-core)-compatible proxy
configurations.

<p align="center">
  <img src="docs/src/media/screenshot.png" alt="XRAT terminal UI showing proxy testing progress, config details, logs, and runtime status">
</p>

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/mhyrzt/xrat/master/install.sh | bash
```

Requires `xray` on your system. For other installation methods (manual binary
download, Docker, build from source, shell completions, man pages) see the
[installation docs](https://mhyrzt.github.io/xrat/01-getting-started/installation.html).

## Current Features

- **Import** subscriptions, files, raw links, base64 lists, SIP008 JSON, and
  Xray JSON into SQLite or PostgreSQL.
- **Support** VLESS, VMess, Trojan, Shadowsocks, HTTP, SOCKS5, and Hysteria2
  parsing/preview.
- **Deduplicate** configs with normalized, versioned keys while preserving
  subscription metadata.
- **Test** proxies with ICMP, TCP, real-delay, download, and upload stages.
- **Rank** bulk test results with concurrency control, failure classification,
  history, and GeoIP enrichment.
- **Run** stored configs as managed Xray-core or V2Ray-core local proxy
  sessions.
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

## Acknowledgments

Some functionalities in XRAT have been inspired by
[xray-knife](https://github.com/lilendian0x00/xray-knife).
