<p align="center">
  <img src="media/xrat-hero.png" alt="xrat" width="600">
</p>

<h1 align="center">XRAT</h1>

<p align="center">
  <em>A modern proxy configuration manager for Xray-core and sing-box</em>
</p>

<p align="center">
  <img alt="Status" src="https://img.shields.io/badge/status-under%20development-orange">
  <img alt="Rust" src="https://img.shields.io/badge/rust-stable-blue">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green">
</p>

XRAT is a Rust CLI for importing, storing, testing, and running Xray-compatible
proxy configurations.

## Current Features

- Import subscription/config lines into SQLite or PostgreSQL.
- List stored configs and subscriptions.
- Test configs for connectivity, latency, and optional download speed.
- Run one stored config as a managed local Xray runtime.
- Show runtime status and stop the active runtime session.
- Parse and validate links without importing, with optional JSON output.

## Development

Common commands:

```bash
cargo build
cargo test -q
cargo fmt
```

Run the CLI locally:

```bash
cargo run -- import <input>
cargo run -- list configs
cargo run -- test
cargo run -- connect <id>
cargo run -- status
cargo run -- disconnect
cargo run -- parse 'vless://...'
cargo run -- parse --json --engine auto 'vless://...'
cargo run -- parse --json --engine auto 'hy2://...'
```

## Documentation

Full documentation is available at [`docs/src/`](docs/src/).
