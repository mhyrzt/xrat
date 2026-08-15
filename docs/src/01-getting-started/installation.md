# Installation Script

Use the installer script for a normal Linux or macOS install. It downloads the
matching release archive, verifies the checksum, installs `xrat`, and can run
first-time setup for you.

For other install paths, see [Docker Install](docker-install.md),
[Manual Binary Install](manual-binary-install.md), or
[Build From Source](source-install.md).

## Requirements

### Runtime dependencies

| Tool       | Required | Purpose                                                 | Upstream                                                  |
| ---------- | -------- | ------------------------------------------------------- | --------------------------------------------------------- |
| `xray`     | Yes      | Managed Xray runtime and real-delay tests               | [XTLS/Xray-core](https://github.com/XTLS/Xray-core)       |
| `sing-box` | No       | sing-box preview and managed Hysteria2 runtime sessions | [SagerNet/sing-box](https://github.com/SagerNet/sing-box) |
| `v2ray`    | No       | Alternative V2Ray managed runtime                       | [V2Fly/V2Ray](https://github.com/v2fly/v2ray-core)        |

`xrat setup` detects these tools, checks their latest stable versions, and can
install verified user-local copies without root access. Managed files live
under `~/.local/share/xrat/cores`, with commands linked into `~/.local/bin`.
Existing system or package-manager installations are never overwritten.

The upstream system installers remain available when a system-wide service is
preferred. Install Xray system-wide:

```bash
bash -c "$(curl -L https://github.com/XTLS/Xray-install/raw/main/install-release.sh)" @ install
```

Install sing-box if you need Hysteria2 (`hy2`) managed runtime support:

```bash
curl -fsSL https://sing-box.app/install.sh | sh
```

Install V2Ray system-wide on a supported systemd Linux distribution:

```bash
bash -c "$(curl -L https://raw.githubusercontent.com/v2fly/fhs-install-v2ray/master/install-release.sh)"
```

### System requirements

| Requirement | Details                                           |
| ----------- | ------------------------------------------------- |
| OS          | Linux x86_64/aarch64, or macOS x86_64/arm64       |
| libc        | None -- Linux release binaries are statically linked |
| SQLite      | Bundled -- no system SQLite needed                |
| PostgreSQL  | Optional -- version 14+ if used instead of SQLite |
| Network     | Outbound HTTPS for imports and release downloads  |

## Platform Support

Core CLI, config import, parsing, testing, and the TUI work on any Unix-like
platform xrat compiles for. Platform integrations vary:

| Feature                | Linux          | macOS            | FreeBSD          | OpenBSD          |
| ---------------------- | -------------- | ---------------- | ---------------- | ---------------- |
| CLI / config / import  | yes            | yes              | expected         | expected         |
| daemon runtime IPC     | Unix socket    | Unix socket      | Unix socket      | Unix socket      |
| daemon install         | systemd user   | launchd agent    | rc.d (root)      | rc.d (root)      |
| runtime reattach       | sysinfo        | sysinfo          | sysinfo          | sysinfo (cmd)    |
| desktop proxy          | GNOME/gsettings| networksetup     | unsupported      | unsupported      |
| release upgrade        | musl tarball   | darwin tarball   | source/manual    | source/manual    |
| clipboard (TUI)        | X11/Wayland    | native           | X11              | X11              |

macOS and BSD integrations are newer; the FreeBSD/OpenBSD rows are expected to
work but are not yet verified on hardware. Windows is tracked separately and not
yet supported.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/mhyrzt/xrat/master/install.sh | bash
```

To run all setup prompts with yes answers:

```bash
curl -fsSL https://raw.githubusercontent.com/mhyrzt/xrat/master/install.sh | bash -s -- --yes
```

The installer will:

1. Detect the OS and architecture and pick the release target triple.
2. Download the latest GitHub release archive.
3. Verify the archive against `SHASUMS256.txt` (`sha256sum` or `shasum`).
4. Install `xrat` to `~/.local/bin/xrat`.
5. Hand off to [`xrat setup`](../02-cli/setup.md) for post-install setup:
   managed dependency checks and optional installs, `xrat init`, the background
   daemon, shell completions, man pages, an `xratui` shortcut, and (Linux/XDG)
   the desktop launcher and icons.

Setup runs in the binary, so it works the same regardless of how xrat was
installed and can be re-run any time with `xrat setup`. See the
[setup reference](../02-cli/setup.md) for the full step list and `--check`
diagnostics.

Useful flags (passed through to `xrat setup`):

| Flag                | Purpose                                                  |
| ------------------- | -------------------------------------------------------- |
| `--from-source`     | Build from the current checkout instead of downloading   |
| `--install-dir DIR` | Binary install directory                                |
| `--no-desktop`      | Skip installing desktop launcher and icon assets         |
| `--linger`          | Enable boot-before-login daemon start (Linux)            |
| `-y`, `--yes`       | Skip prompts and accept setup defaults                   |
| `-h`, `--help`      | Show installer help                                      |

To install to a different directory:

```bash
curl -fsSL https://raw.githubusercontent.com/mhyrzt/xrat/master/install.sh | bash -s -- --install-dir /usr/local/bin
```

To skip the desktop launcher:

```bash
curl -fsSL https://raw.githubusercontent.com/mhyrzt/xrat/master/install.sh | bash -s -- --no-desktop
```

The desktop launcher starts the TUI in a detected terminal emulator. When the
installer finds a supported terminal, it generates a launcher that sets xrat's
window identity for taskbar/dock icon matching on X11 or Wayland. If no
supported terminal is found, the launcher falls back to the desktop's default
terminal behavior and the taskbar icon may belong to that terminal window.

| Terminal | X11 identity | Wayland identity | Notes |
| -------- | ------------ | ---------------- | ----- |
| kitty | `--class=xrat` | `--class=xrat` / app id | Preferred cross-session launcher |
| Alacritty | `--class xrat,xrat` | `--class xrat,xrat` | Preferred cross-session launcher |
| WezTerm | `--class xrat` | `--class xrat` / app id | Preferred cross-session launcher |
| foot / footclient | n/a | `--app-id=xrat` | Wayland-only terminal |
| Konsole | `--desktopfile xrat` | `--desktopfile xrat` | KDE/Qt desktop-file identity hint |
| GNOME Terminal | `--class=xrat` | fallback only | Used for X11 sessions |
| xterm | `-class xrat` | n/a | X11-only fallback |

Make sure the install directory is in `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Add that line to `~/.bashrc`, `~/.zshrc`, or your shell's equivalent startup
file if needed.

## Build and Install From Local Checkout

Pass `--from-source` to have the installer build the binary from the repository
instead of downloading a release archive. Run the script directly from the repo
root — piping from `curl` will not work because the script needs `Cargo.toml`
present alongside it.

Requirements: `cargo` must be in `PATH`. `git`, `curl`, `tar`, and `sha256sum`
are not needed.

```bash
git clone https://github.com/mhyrzt/xrat.git
cd xrat
bash install.sh --from-source
```

To install to a different directory:

```bash
bash install.sh --from-source --install-dir /usr/local/bin
```

To skip prompts:

```bash
bash install.sh --from-source --yes
```

The script will:

1. Run `cargo build --release` inside the checkout.
2. Install `xrat` to the install directory.
3. Hand off to `xrat setup`, which generates man pages, completions, and desktop
   assets from the built binary the same way as the release path.

For a pure Cargo-managed install or a development workflow, see
[Build From Source](source-install.md).

## First-Time Setup

If you installed xrat another way (e.g. `cargo install`, a package manager, or a
manual copy), or skipped the installer's setup, run setup yourself:

```bash
xrat setup
```

This is idempotent and re-runnable, so it also works to finish or repair an
install. Check what is and isn't configured without changing anything:

```bash
xrat setup --check
```

To do just the individual pieces instead: `xrat init` for the config directory
and database, or `xrat daemon install --start` for the background daemon.

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
| `$HOME/.local/share/xrat/cores/` | Managed proxy cores/assets   | XDG data directory |
| `$HOME/.local/bin/{xray,v2ray,sing-box}` | Managed core CLI links | -             |
