# Installation

## Requirements

### Runtime dependencies

| Tool      | Required | Purpose                                   | Install                                                                 |
| --------- | -------- | ----------------------------------------- | ----------------------------------------------------------------------- |
| `xray`    | Yes      | Xray engine, real-delay tests             | [XTLS/Xray-install](https://github.com/XTLS/Xray-install)              |
| `sing-box` | No      | sing-box config parsing and runtime       | [sing-box.app](https://sing-box.app/install.sh)                         |

Install xray:

```bash
bash -c "$(curl -L https://github.com/XTLS/Xray-install/raw/main/install-release.sh)" @ install
```

Install sing-box (optional):

```bash
curl -fsSL https://sing-box.app/install.sh | sh
```

### System requirements

| Requirement | Details                                           |
| ----------- | ------------------------------------------------- |
| OS          | Linux x86_64 or aarch64                          |
| libc        | None — binaries are statically linked (musl)     |
| SQLite      | Bundled — no system SQLite needed                |
| PostgreSQL  | Optional — version 14+ if used instead of SQLite |
| Network     | Outbound HTTPS for imports                        |

---

## Option 1 — Installer script (recommended)

Downloads the correct release binary for your architecture, verifies the
checksum, installs to `~/.local/bin/`, and optionally runs first-time setup.

```bash
curl -fsSL https://raw.githubusercontent.com/mhyrzt/xrat/master/install.sh | bash
```

The installer will:

1. Check for `xray` (required) and `sing-box` (optional).
2. Download and verify the latest release binary.
3. Install to `~/.local/bin/xrat` (override with `INSTALL_DIR=/usr/local/bin`).
4. Offer to run `xrat init` and set up the systemd daemon.

To install to a custom directory:

```bash
INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/mhyrzt/xrat/master/install.sh | bash
```

---

## Option 2 — Manual binary download

1. Go to [Releases](https://github.com/mhyrzt/xrat/releases/latest) and download
   the archive for your architecture:

   | File                                       | Architecture      |
   | ------------------------------------------ | ----------------- |
   | `xrat-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz`  | x86_64 (most PCs) |
   | `xrat-vX.Y.Z-aarch64-unknown-linux-musl.tar.gz` | ARM64 (Pi 4/5, Graviton) |

2. Verify the checksum:

   ```bash
   sha256sum -c SHASUMS256.txt --ignore-missing
   ```

3. Extract and install:

   ```bash
   tar -xzf xrat-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz
   mkdir -p ~/.local/bin
   mv xrat ~/.local/bin/xrat
   chmod +x ~/.local/bin/xrat
   ```

4. Ensure `~/.local/bin` is in your `PATH`:

   ```bash
   export PATH="$HOME/.local/bin:$PATH"   # add to ~/.bashrc or ~/.zshrc
   ```

### Shell completions (included in archive)

```bash
# Bash
cp completions/xrat.bash ~/.local/share/bash-completion/completions/xrat

# Zsh
cp completions/_xrat ~/.zfunc/_xrat

# Fish
cp completions/xrat.fish ~/.config/fish/completions/xrat.fish
```

### Man pages (included in archive)

```bash
mkdir -p ~/.local/share/man/man1
cp man/man1/*.1 ~/.local/share/man/man1/
mandb ~/.local/share/man
```

### Desktop entry (included in archive)

```bash
cp desktop/xrat.desktop ~/.local/share/applications/
mkdir -p ~/.local/share/icons/hicolor/48x48/apps
mkdir -p ~/.local/share/icons/hicolor/256x256/apps
cp desktop/icons/xrat-48x48.png ~/.local/share/icons/hicolor/48x48/apps/xrat.png
cp desktop/icons/xrat-256x256.png ~/.local/share/icons/hicolor/256x256/apps/xrat.png
update-desktop-database ~/.local/share/applications/
```

---

## Option 3 — Build from source

Requirements: Rust toolchain via [rustup](https://rustup.rs).

```bash
git clone https://github.com/mhyrzt/xrat.git
cd xrat
cargo install --path . --locked
```

The binary is installed to `~/.cargo/bin/xrat`. Ensure `~/.cargo/bin` is in your `PATH` (rustup does this automatically).

To build a release binary manually:

```bash
cargo build --release --locked
# binary at target/release/xrat
```

---

## First-time setup

After installation, initialize xrat's config directory and database:

```bash
xrat init
```

Then follow the [Quickstart](quickstart.md) to import configs and connect.

---

## State paths

| Path                             | Purpose                      | Override            |
| -------------------------------- | ---------------------------- | ------------------- |
| `$HOME/.config/xrat/`            | App root                     | `XRAT_PATH` env var |
| `$HOME/.config/xrat/config.toml` | Configuration                | `--config` flag     |
| `$HOME/.config/xrat/db.sqlite`   | SQLite database              | `--database` flag   |
| `$HOME/.config/xrat/runtime/`    | Daemon socket, session state | —                   |
| `$HOME/.config/xrat/logs/`       | Runtime logs                 | `[runtime.log].dir` |
| `$HOME/.config/xrat/mmdb/`       | GeoIP data                   | `[mmdb].dir`        |
