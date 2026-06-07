# Installation Script

Use the installer script for a normal Linux install. It downloads the matching
release archive, verifies the checksum, installs `xrat`, and can run first-time
setup for you.

For other install paths, see [Docker Install](docker-install.md),
[Manual Binary Install](manual-binary-install.md), or
[Build From Source](source-install.md).

## Requirements

### Runtime dependencies

| Tool       | Required | Purpose                                                 | Install                                                   |
| ---------- | -------- | ------------------------------------------------------- | --------------------------------------------------------- |
| `xray`     | Yes      | Managed Xray runtime and real-delay tests               | [XTLS/Xray-install](https://github.com/XTLS/Xray-install) |
| `sing-box` | No       | sing-box preview and managed Hysteria2 runtime sessions | [sing-box.app](https://sing-box.app/install.sh)           |

Install xray:

```bash
bash -c "$(curl -L https://github.com/XTLS/Xray-install/raw/main/install-release.sh)" @ install
```

Install sing-box if you need Hysteria2 (`hy2`) managed runtime support:

```bash
curl -fsSL https://sing-box.app/install.sh | sh
```

### System requirements

| Requirement | Details                                           |
| ----------- | ------------------------------------------------- |
| OS          | Linux x86_64 or aarch64                           |
| libc        | None -- release binaries are statically linked    |
| SQLite      | Bundled -- no system SQLite needed                |
| PostgreSQL  | Optional -- version 14+ if used instead of SQLite |
| Network     | Outbound HTTPS for imports and release downloads  |

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/mhyrzt/xrat/master/install.sh | bash
```

The installer will:

1. Check for `xray` and warn if optional `sing-box` is missing.
2. Detect `x86_64` or `aarch64`.
3. Download the latest GitHub release archive.
4. Verify the archive against `SHASUMS256.txt`.
5. Install `xrat` to `~/.local/bin/xrat`.
6. Install bundled man pages, shell completions, and desktop launcher assets.
7. Offer to run `xrat init` when the script has an interactive terminal.
8. Offer to install and start the systemd user daemon by default.
9. Offer to enable systemd user lingering for boot startup before login.

To install to a different directory:

```bash
INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/mhyrzt/xrat/master/install.sh | bash
```

To skip the desktop launcher:

```bash
INSTALL_DESKTOP=0 curl -fsSL https://raw.githubusercontent.com/mhyrzt/xrat/master/install.sh | bash
```

Make sure the install directory is in `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Add that line to `~/.bashrc`, `~/.zshrc`, or your shell's equivalent startup
file if needed.

## Build and Install From Local Checkout

Set `BUILD_FROM_SOURCE=1` to have the installer build the binary from the
repository instead of downloading a release archive. Run the script directly
from the repo root — piping from `curl` will not work because the script needs
`Cargo.toml` present alongside it.

Requirements: `cargo` must be in `PATH`. `git`, `curl`, `tar`, and `sha256sum`
are not needed.

```bash
git clone https://github.com/mhyrzt/xrat.git
cd xrat
BUILD_FROM_SOURCE=1 bash install.sh
```

To install to a different directory:

```bash
BUILD_FROM_SOURCE=1 INSTALL_DIR=/usr/local/bin bash install.sh
```

The script will:

1. Run `cargo build --release` inside the checkout.
2. Generate man pages and shell completions from the built binary.
3. Install `xrat`, man pages, completions, and desktop launcher assets the same
   way as the release path.
4. Offer first-time setup prompts.

For a pure Cargo-managed install or a development workflow, see
[Build From Source](source-install.md).

## First-Time Setup

If you skipped the installer's setup prompts, initialize xrat manually:

```bash
xrat init
```

To install the daemon later:

```bash
xrat daemon install --start
```

Then follow the [Quickstart](quickstart.md) to import configs and connect.

## State Paths

| Path                             | Purpose                      | Override            |
| -------------------------------- | ---------------------------- | ------------------- |
| `$HOME/.config/xrat/`            | App root                     | `XRAT_PATH` env var |
| `$HOME/.config/xrat/config.toml` | Configuration                | `--config` flag     |
| `$HOME/.config/xrat/db.sqlite`   | SQLite database              | `--database` flag   |
| `$HOME/.config/xrat/runtime/`    | Daemon socket, session state | -                   |
| `$HOME/.config/xrat/logs/`       | Runtime logs                 | `[runtime.log].dir` |
| `$HOME/.config/xrat/mmdb/`       | GeoIP data                   | `[mmdb].dir`        |
