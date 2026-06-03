<p align="center">
  <img src="media/icons/xrat-icon-1024x1024.png" alt="xrat" width="256">
</p>

<h1 align="center">XRAT</h1>

<p align="center">
  <em>A modern proxy configuration manager for Xray-core and sing-box</em>
</p>

<p align="center">
  <img alt="Status" src="https://img.shields.io/badge/status-under%20development-orange">
  <img alt="Rust" src="https://img.shields.io/badge/rust-stable-blue">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green">
  <a href="https://mhyrzt.github.io/xrat"><img src="https://img.shields.io/badge/docs-mhyrzt.github.io%2Fxrat-blue" alt="Documentation"></a>
  <a href="https://crates.io/crates/xrat"><img src="https://img.shields.io/crates/v/xrat" alt="crates.io"></a>
</p>

XRAT is a Rust CLI for importing, storing, testing, and running Xray-compatible
proxy configurations.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/mhyrzt/xrat/master/install.sh | bash
```

Requires `xray` on your system. For other installation methods (manual binary
download, Docker, build from source, shell completions, man pages) see the
[installation docs](https://mhyrzt.github.io/xrat/01-getting-started/installation.html).

## Current Features

- Import subscription/config lines into SQLite or PostgreSQL.
- List stored configs and subscriptions.
- Test configs for connectivity, latency, and optional download speed.
- Run one stored config as a managed local Xray runtime.
- Show runtime status and stop the active runtime session.
- Parse and validate links without importing, with optional JSON output.

## Acknowledgments

Some functionalities in XRAT have been inspired by
[xray-knife](https://github.com/lilendian0x00/xray-knife).
