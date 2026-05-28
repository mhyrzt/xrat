# Getting Started

xrat is a Rust-based CLI tool and daemon for managing proxy configurations. It
imports subscription links, parses and normalizes proxy URIs, tests connectivity
and performance, generates runtime configs for Xray-core and sing-box, manages a
local proxy runtime process, and exposes an HTTP API.

## Prerequisites

- **Rust toolchain** (for building from source): `rustup` or Rust 1.85+
- **Xray-core** or **V2Ray** binary installed and available in `PATH`
- **sing-box** binary (optional, required for Hysteria2 support)

## Installation

### Building from Source

```bash
git clone <repository-url>
cd xrat
cargo build --release
```

The compiled binary will be at `target/release/xrat`.

### Development Build

```bash
cargo build
cargo run -- <command>
```

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
- [Configuration](configuration.md) — config.toml reference
