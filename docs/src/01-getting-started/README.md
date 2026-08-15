# Getting Started

xrat is a Rust-based CLI tool and daemon for managing proxy configurations. It
imports subscription links, parses and normalizes proxy URIs, tests connectivity
and performance, previews runtime configs for Xray-core and sing-box, manages an
Xray/V2Ray local proxy runtime process plus sing-box-backed Hysteria2 sessions,
and exposes an HTTP API.

## Prerequisites

- **Xray-core** binary installed and available in `PATH`
- **Proxy cores**: Xray is required for runtime use; setup can install Xray,
  sing-box, and V2Ray as verified user-local tools
- **Rust toolchain** and **just** when building from source

## Installation

Choose one install path:

- [Installation Script](installation.md) — recommended Linux install from the
  latest verified release archive
- [Docker Install](docker-install.md) — run the published container image with
  bundled Xray-core
- [Manual Binary Install](manual-binary-install.md) — download, verify, and
  place release files yourself
- [Cargo Install](cargo-install.md) — `cargo install xrat` from crates.io
- [Build From Source](source-install.md) — Justfile-oriented workflow for local
  development builds and source installs

## Configuration Directory

xrat uses a configuration directory with the following resolution order:

1. `--config <path>` CLI flag
2. `XRAT_PATH` environment variable
3. `~/.config/xrat/`

The directory layout:

```
~/.config/xrat/
├── config.toml      # Application configuration
├── db.sqlite        # SQLite database (default)
├── runtime/         # Runtime session files (generated configs, logs)
└── logs/            # Xray/V2Ray process logs
```

## Next Steps

- [Quickstart](quickstart.md) — import, test, and connect in 3 commands
- [Installation Script](installation.md) — recommended install path
- [Configuration](configuration.md) — config.toml reference
