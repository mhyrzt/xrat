# Phase 7+8: Release Readiness, Desktop Integration, and Distribution

This document combines the installation/packaging work (Phase 8) and the desktop
integration work (Phase 7) into a single phase focused on making xrat adoptable,
installable, and accessible outside a development checkout.

Each section is scoped to a self-contained deliverable with explicit
implementation targets grounded in the current codebase.

Target platform: **Linux** (primary). Windows binary builds are produced by CI
but have limited testing and no platform-specific integration.

> **Completed:** `xrat daemon install/uninstall`, `xrat init`, and
> `xrat manpage` are fully implemented. `--version` flag wired up. Remaining
> work starts at section 1 below.

---

## 1. Prerequisites and Installation Documentation

### Current State

No dedicated installation page exists. Installation instructions are scattered
across:

- `docs/src/01-getting-started/quickstart.md` — assumes the binary is already
  available
- `docs/src/04-deployment/README.md` — deployment checklist, no install steps
- `docs/src/04-deployment/systemd.md` — manual systemd setup (393 lines)
- `docs/src/README.md` — project overview, no install steps

### Required Documentation

Create `docs/src/01-getting-started/installation.md` covering:

**System requirements**:

| Requirement | Details                                                                        |
| ----------- | ------------------------------------------------------------------------------ |
| OS          | Linux (primary), Windows (binary builds, limited testing)                      |
| Rust/Cargo  | Required for source builds; `rustup` recommended                               |
| SQLite      | Bundled via `sqlx`; no system SQLite needed                                    |
| PostgreSQL  | Optional; version 14+ when used                                                |
| Network     | Outbound HTTPS for imports; local ports 1080 (SOCKS), 8080 (HTTP) for inbounds |

**Runtime binaries** (resolved via CLI flags → `config.toml` `[paths]` →
`$PATH`):

| Binary     | Required for                  | Config key                         |
| ---------- | ----------------------------- | ---------------------------------- |
| `xray`     | Xray engine, real-delay tests | `[paths].xray` or `--xray`         |
| `v2ray`    | V2Ray engine                  | `[paths].v2ray` or `--v2ray`       |
| `sing-box` | sing-box parsing/runtime      | `[paths].sing_box` or `--sing-box` |

At least one engine binary must be available for proxy runtime features. Import,
parse, list, and test (except real-delay) work without any engine binary.

**Installation methods**:

1. From release binary (once published):
   ```bash
   tar xzf xrat-<version>-x86_64-unknown-linux-gnu.tar.gz
   mv xrat ~/.local/bin/
   ```
2. From source:
   ```bash
   git clone https://github.com/mhyrzt/xrat.git
   cd xrat
   cargo install --path .
   ```
3. Verify: `xrat --version`

**State paths**:

| Path                             | Purpose                      | Override            |
| -------------------------------- | ---------------------------- | ------------------- |
| `$HOME/.config/xrat/`            | App root                     | `XRAT_PATH` env var |
| `$HOME/.config/xrat/config.toml` | Configuration                | `--config` flag     |
| `$HOME/.config/xrat/db.sqlite`   | SQLite database              | `--database` flag   |
| `$HOME/.config/xrat/runtime/`    | Daemon socket, session state | —                   |
| `$HOME/.config/xrat/logs/`       | Runtime logs                 | `[runtime.log].dir` |
| `$HOME/.config/xrat/mmdb/`       | GeoIP data                   | `[mmdb].dir`        |

**First-time setup**:

```bash
xrat init                    # Create config, database, directories
xrat import <url>            # Import a subscription
xrat list configs            # Verify imported configs
xrat test --selected-only    # Test connectivity
xrat connect <id>            # Start proxy
```

### Documentation Structure Changes

1. Create `docs/src/01-getting-started/installation.md` — the page described
   above.
2. Update `docs/src/SUMMARY.md` to add Installation before Quickstart:
   ```
   - [Installation](01-getting-started/installation.md)
   - [Quickstart](01-getting-started/quickstart.md)
   ```
3. Update `docs/src/01-getting-started/quickstart.md` to begin with `xrat init`
   and link to the installation page.
4. Update `docs/src/04-deployment/systemd.md`: move manual service file setup to
   a "Reference: Manual Setup" section; make `xrat daemon install` the primary
   path.

### Definition of Done

- A new user can identify prerequisites, install xrat, initialize state, and run
  the quickstart without reading architecture docs.
- Documentation clearly separates: source build setup, binary install setup,
  daemon service setup, and optional PostgreSQL setup.
- All state paths and override mechanisms are documented in one place.

---

## 2. Shell Completion

### Current State

No shell completion support exists. The CLI is defined via Clap 4.6.1 with
`derive` feature in `src/cli/`. Clap provides `clap_complete` for generating
completions from the command tree.

### Implementation

Add `clap_complete` to `Cargo.toml`:

```toml
clap_complete = "4"
```

Add a hidden `completions` command to `src/cli/command.rs`:

```rust
Completions(CompletionsArgs),
```

```rust
#[derive(Args)]
struct CompletionsArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    shell: clap_complete::Shell,
}
```

The handler in `src/app/commands/completions.rs` calls
`clap_complete::generate()` with the root Clap command and writes to stdout.

### Usage

```bash
# Bash
xrat completions bash > ~/.local/share/bash-completion/completions/xrat

# Zsh
xrat completions zsh > ~/.zfunc/_xrat

# Fish
xrat completions fish > ~/.config/fish/completions/xrat.fish

# PowerShell
xrat completions powershell > xrat.ps1
```

### Packaging Integration

Release archives should include pre-generated completion scripts:

- `completions/xrat.bash`
- `completions/_xrat` (zsh)
- `completions/xrat.fish`

CI generates these during the release workflow using `xrat completions <shell>`
and includes them in the release tarball.

### Definition of Done

- `xrat completions bash` generates a working bash completion script.
- Completions cover all commands, subcommands, and flags.
- Completion output updates automatically when CLI changes.
- Release artifacts include pre-generated completion scripts.
- Documentation covers installation for each shell.

---

## 3. Release Workflow Improvements

### Current State

The existing `.github/workflows/release.yml` triggers on `v*` tags and builds
release binaries for three platforms:

| Runner           | Artifact name             |
| ---------------- | ------------------------- |
| `ubuntu-latest`  | `xrat-linux-x86_64`       |
| `windows-latest` | `xrat-windows-x86_64.exe` |

Build command: `cargo build --release --locked`. Artifacts are packaged into
`dist/` and published via `gh release create`.

The CI workflow (`.github/workflows/ci.yml`) runs `cargo fmt --check`,
`cargo clippy --all-targets --all-features`, and `cargo test -q`.

### Workflow Changes

**Pre-release validation** (add to `release.yml` before build):

1. `cargo fmt --check` — fail if code is not formatted.
2. `cargo clippy --all-targets --all-features -- -D warnings` — fail on
   warnings.
3. `cargo test -q` — fail if tests do not pass.
4. Verify `Cargo.lock` is not stale: `cargo build --locked --release` already
   handles this.

**Artifact naming** — use target triple names for clarity:

| Current                   | Proposed                                         |
| ------------------------- | ------------------------------------------------ |
| `xrat-linux-x86_64`       | `xrat-<version>-x86_64-unknown-linux-gnu.tar.gz` |
| `xrat-windows-x86_64.exe` | `xrat-<version>-x86_64-pc-windows-msvc.zip`      |

Each archive should contain:

- `xrat` binary
- `LICENSE` (MIT)
- `README.md`
- `completions/` (pre-generated shell completions)
- `man/` (pre-generated man pages, Linux only)
- `packaging/systemd/` (service files, Linux only)

**Checksums**: generate SHA-256 checksums for every artifact and publish
`SHASUMS256.txt` alongside the release.

**Release notes**: use `gh release create --generate-notes` or maintain a
`CHANGELOG.md` with conventional commit summaries. Replace the generic one-line
release description.

**Additional targets to evaluate**:

| Target                      | Use case                                 | Priority |
| --------------------------- | ---------------------------------------- | -------- |
| `x86_64-unknown-linux-musl` | Static Linux binary, no glibc dependency | High     |
| `aarch64-unknown-linux-gnu` | ARM64 Linux (Raspberry Pi, AWS Graviton) | Medium   |

### Package Formats to Evaluate

| Format         | Target audience            | Effort                                              | Priority |
| -------------- | -------------------------- | --------------------------------------------------- | -------- |
| `.deb`         | Debian/Ubuntu users        | Medium (use `cargo-deb` or manual `DEBIAN/control`) | High     |
| `.rpm`         | Fedora/RHEL/openSUSE users | Medium (use `cargo-generate-rpm`)                   | Medium   |
| AUR `PKGBUILD` | Arch Linux users           | Low (source-based, just a recipe)                   | Medium   |
| Nix derivation | NixOS/Nix users            | Low                                                 | Low      |

Each package should:

- Install the `xrat` binary to the standard bin directory.
- Include shell completions in the correct completion directory.
- Include the man page in the standard man directory.
- NOT create or overwrite user state (`~/.config/xrat/`). Users still run
  `xrat init`.
- Include license and README.

### Release Readiness Checklist

- [ ] Version in `Cargo.toml` matches the tag name (strip leading `v`).
- [ ] `cargo fmt`, `cargo clippy`, `cargo test` all pass in CI before packaging.
- [ ] Release artifacts use target-triple naming with version prefix.
- [ ] Every artifact has a SHA-256 checksum published in `SHASUMS256.txt`.
- [ ] Archives include `LICENSE`, `README.md`, completions, and man pages.
- [ ] Package installs do not touch user state.
- [ ] Smoke test: `xrat --version`, `xrat init --dry-run`, `xrat --help` all
      work from the packaged binary.
- [ ] Documentation build (`docs.yml`) passes for the release tag.
- [ ] Release notes summarize changes since the last release.

---

## 4. Desktop Entry (Linux)

### Current State

No `.desktop` files exist in the repository. No desktop integration code exists.
Pre-sized icon assets are available in `media/icons/`.

### Specification

A Freedesktop-compliant `.desktop` file:

```ini
[Desktop Entry]
Type=Application
Name=XRAT
Comment=Proxy/VPN configuration manager and runtime
Exec=xrat tui
Terminal=true
Categories=Network;
Keywords=proxy;vpn;xray;sing-box;tui;
Icon=xrat
StartupNotify=false
```

Key considerations:

- **`Terminal=true`** — xrat is a TUI and requires a terminal emulator. Some
  launchers handle this correctly; others silently fail. If launcher support is
  inconsistent, a wrapper script may be needed:
  `Exec=sh -c 'exec "$TERMINAL" -e xrat tui || exec xterm -e xrat tui'`.
- **`StartupNotify=false`** — TUI applications do not participate in the startup
  notification protocol.

### Icon Assets

Pre-sized icons are available in `media/icons/`. Use the appropriate sizes for
the desktop entry:

| Source file                         | Size    | Install path                                         |
| ----------------------------------- | ------- | ---------------------------------------------------- |
| `media/icons/xrat-icon-48x48.png`   | 48x48   | `~/.local/share/icons/hicolor/48x48/apps/xrat.png`   |
| `media/icons/xrat-icon-64x64.png`   | 64x64   | `~/.local/share/icons/hicolor/64x64/apps/xrat.png`   |
| `media/icons/xrat-icon-128x128.png` | 128x128 | `~/.local/share/icons/hicolor/128x128/apps/xrat.png` |
| `media/icons/xrat-icon-256x256.png` | 256x256 | `~/.local/share/icons/hicolor/256x256/apps/xrat.png` |

System-wide installs use `/usr/share/icons/hicolor/...` instead.

After installation, run (non-fatal if missing):

- `update-desktop-database ~/.local/share/applications/`
- `gtk-update-icon-cache ~/.local/share/icons/hicolor/`

### Definition of Done

- `xrat integrate` installs a working `.desktop` entry and icon assets on Linux.
- xrat appears in the system app launcher after installation.
- Documentation covers the desktop entry in install docs.

---

## 5. `xrat integrate` Command

### CLI Changes

Add a new top-level command:

```rust
Integrate(IntegrateArgs),
```

```rust
#[derive(Args)]
struct IntegrateArgs {
    /// Print planned actions without executing them
    #[arg(long)]
    dry_run: bool,

    /// Remove installed desktop integration files
    #[arg(long)]
    uninstall: bool,
}
```

### Implementation

Add `src/app/commands/integrate.rs`:

**Install mode** (default):

1. Generate `xrat.desktop` with the resolved binary path in `Exec=`.
2. Write to `~/.local/share/applications/xrat.desktop`.
3. Copy pre-sized icons from `media/icons/` (48x48, 64x64, 128x128, 256x256) and
   write to `~/.local/share/icons/hicolor/<size>x<size>/apps/xrat.png`.
   - Embed pre-sized PNGs in the binary at compile time using `include_bytes!`
     and a `build.rs` step, or read from the source tree at runtime for
     development installs.
4. Run `update-desktop-database` and `gtk-update-icon-cache` if available
   (non-fatal if missing).
5. Print summary of installed files.

**Uninstall mode** (`--uninstall`):

1. Remove `~/.local/share/applications/xrat.desktop`.
2. Remove all `~/.local/share/icons/hicolor/*/apps/xrat.png`.
3. Print summary of removed files.

**Platform detection**: return `UnsupportedPlatform` on non-Linux.

### Definition of Done

- `xrat integrate` installs desktop entry and icons on Linux.
- `xrat integrate --uninstall` removes them cleanly.
- `xrat integrate --dry-run` prints planned actions.
- CLI parser tests cover the command and flags.

---

## 6. Documentation Updates

### Pages to Create

| Path                                          | Content                                          |
| --------------------------------------------- | ------------------------------------------------ |
| `docs/src/01-getting-started/installation.md` | Prerequisites, install methods, first-time setup |

### Pages to Update

| Path                                        | Changes                                                            |
| ------------------------------------------- | ------------------------------------------------------------------ |
| `docs/src/SUMMARY.md`                       | Add Installation page before Quickstart                            |
| `docs/src/01-getting-started/quickstart.md` | Begin with `xrat init`; link to installation                       |
| `docs/src/02-cli/daemon.md`                 | Add `install`/`uninstall` command reference                        |
| `docs/src/04-deployment/systemd.md`         | Make `xrat daemon install` primary; manual setup becomes reference |
| `docs/src/02-cli/`                          | Add `init.md`, `integrate.md`, `tray.md` command pages             |

### Platform-Specific Notes

Document in installation or tray docs:

- **Wayland**: `libayatana-appindicator` has inconsistent Wayland support. On
  GNOME Wayland, tray icons require the AppIndicator extension or
  KStatusNotifierItem protocol. Document this caveat.
- **X11**: tray icons work out of the box with `libayatana-appindicator`.

---

## 7. System Notifications (Optional)

### Dependency

```toml
notify-rust = { version = "4", optional = true }

[features]
notifications = ["dep:notify-rust", "tray"]
```

### Implementation

The tray process sends notifications on daemon state transitions:

| Transition     | Notification title | Body                         |
| -------------- | ------------------ | ---------------------------- |
| → Connected    | "xrat"             | "Connected to <config-name>" |
| → Disconnected | "xrat"             | "Disconnected"               |
| → Error        | "xrat"             | "Runtime error: <message>"   |

Rate limiting:

- Maximum one notification per 30 seconds.
- Do not notify on initial state query (only on transitions between states).
- Do not notify if the previous notification was for the same state.

### Linux System Dependencies

`notify-rust` requires `libnotify`:

| Package           | Distros        |
| ----------------- | -------------- |
| `libnotify-dev`   | Ubuntu, Debian |
| `libnotify-devel` | Fedora         |

### Definition of Done

- System notifications appear on connect/disconnect/error events.
- Notification rate is limited to prevent spam.
- Feature is optional and does not affect builds without `notify-rust`.

---

## 8. Tray Icon (Lowest Priority)

> This section is intentionally last. All other deliverables in this phase
> (documentation, shell completion, man pages, release workflow, desktop entry,
> integrate command, notifications) should be completed before starting tray
> icon work.

### Current State

No tray-related code, dependencies, or system integration exists. No Cargo
feature flags are defined — the project compiles as a single monolithic binary.

### Approach

Use the [`tray-icon`](https://crates.io/crates/tray-icon) crate (version 0.19+).
On Linux it wraps `libayatana-appindicator` (modern, Ubuntu 20.10+, Debian 11+,
Fedora) or `libappindicator` (legacy), communicating via D-Bus
StatusNotifierItem protocol.

### Dependencies

```toml
[dependencies]
tray-icon = { version = "0.19", optional = true }

[features]
tray = ["dep:tray-icon"]
```

Feature-gating ensures headless builds and CI can compile without the system
library requirement.

### Architecture: Standalone `xrat tray` Command

A standalone tray process is simpler and more reliable than embedding in the
TUI. The TUI's alternate screen does not survive backgrounding, and terminal
reattachment is platform-specific and fragile.

```bash
xrat tray [--daemon]
```

- `--daemon`: fork to background after initialization.

### CLI Changes

Add to `src/cli/command.rs`:

```rust
Tray(TrayArgs),
```

```rust
#[derive(Args)]
struct TrayArgs {
    /// Fork to background after initialization
    #[arg(long)]
    daemon: bool,
}
```

### Implementation: `src/app/commands/tray.rs`

1. **Initialize tray icon**:
   - Load the default icon from embedded PNG bytes (via `include_bytes!`) or
     from a file path.
   - Create a `tray-icon::TrayIconBuilder` with a context menu.

2. **Context menu**:

   | Item       | Action                                                                             |
   | ---------- | ---------------------------------------------------------------------------------- |
   | Show TUI   | Spawn `xrat tui` in a terminal emulator (detect `$TERMINAL`, fall back to `xterm`) |
   | Connect    | Send `RuntimeConnect { config_id }` via daemon IPC to `<runtime_dir>/daemon.sock`  |
   | Disconnect | Send `RuntimeDisconnect` via daemon IPC                                            |
   | Status     | Display current state as a disabled/read-only menu label                           |
   | Quit       | Exit the tray process                                                              |

3. **Daemon IPC integration**:
   - Connect to the Unix domain socket at `<runtime_dir>/daemon.sock`.
   - Use the existing IPC wire format: JSON, newline-delimited, protocol
     version 1.
   - Poll `RuntimeStatus` on a 5-second `tokio::time::interval`.
   - Parse the response payload to determine state:

     | Daemon response              | Tray icon state             |
     | ---------------------------- | --------------------------- |
     | Connection refused / timeout | Grey — "daemon not running" |
     | `runtime_owned: false`       | Grey — "idle"               |
     | `runtime.status: "starting"` | Yellow — "connecting"       |
     | `runtime.status: "running"`  | Green — "connected"         |
     | `runtime.status: "stopping"` | Yellow — "disconnecting"    |
     | `runtime.status: "error"`    | Red — "error"               |

   - On "Connect" menu click: send `RuntimeConnect` with the currently selected
     config ID (query via `RuntimeStatus` response's `active_config_id`, or show
     a submenu of available configs).
   - On "Disconnect" menu click: send `RuntimeDisconnect`.

4. **Single instance enforcement**:
   - Write PID to `$XDG_RUNTIME_DIR/xrat-tray.pid` (fall back to
     `<runtime_dir>/tray.pid`).
   - On startup, check if the PID file exists and the referenced process is
     alive (`/proc/<pid>/` on Linux).
   - If already running, print "tray already running (PID <n>)" and exit with
     code 1.
   - Remove PID file on clean shutdown.

5. **Event loop**:

   ```
   tokio::select! {
       event = tray_menu_rx.recv() => handle_menu_click(event),
       _ = status_interval.tick() => poll_daemon_status(),
       signal = sigterm_or_sigint() => clean_shutdown(),
   }
   ```

### Tray Icon Assets

Use `media/icons/xrat-icon-32x32.png` as the base tray icon (closest to the
22x22 Linux tray standard). Generate status variants at 22x22:

| Variant      | Description           | File                         |
| ------------ | --------------------- | ---------------------------- |
| Default      | Base icon             | `xrat-tray-default.png`      |
| Connected    | Green tint or overlay | `xrat-tray-connected.png`    |
| Disconnected | Grey tint             | `xrat-tray-disconnected.png` |
| Error        | Red tint              | `xrat-tray-error.png`        |

`tray-icon` loads icons from PNG bytes at runtime, so variant switching is done
by loading different PNG buffers into the `TrayIcon` instance.

### Linux System Dependencies

| Package                        | Distros                           |
| ------------------------------ | --------------------------------- |
| `libayatana-appindicator3-dev` | Ubuntu 20.10+, Debian 11+, Fedora |
| `libappindicator3-dev`         | Legacy distros                    |

The Cargo crate links against whichever is found via `pkg-config`.

### Definition of Done

- `xrat tray` starts a system tray icon with a context menu.
- The tray icon reflects the current daemon/runtime status via IPC polling.
- The tray menu can connect/disconnect the proxy via daemon IPC.
- Single instance enforcement prevents duplicate tray processes.
- `cargo build --features tray` succeeds on Linux (with system deps).
- `cargo build` (without `tray` feature) succeeds without system deps.

---

## Open Questions

1. **Wayland tray compatibility** — Should the tray feature implement a D-Bus
   StatusNotifierItem directly instead of relying on `libayatana-appindicator`?
   The `tray-icon` crate may handle this already; verify before adding
   workarounds.

2. **TUI minimize-to-tray** — Should `xrat tui` support hiding to tray when
   `xrat tray` is running? This requires terminal reattachment, which is
   fragile. Recommendation: defer to a later polish phase.

3. **Auto-start** — Should `xrat tray --autostart` install a desktop autostart
   entry at `~/.config/autostart/xrat-tray.desktop`? Common pattern but adds
   scope. Recommendation: defer; users can create the autostart entry manually.

4. **Tray icon variants** — Should status be communicated by changing icon color
   (requires 3+ PNG assets) or by adding a small overlay badge (requires runtime
   compositing)? Recommendation: pre-generated color variants are simpler and
   avoid a runtime image compositing dependency.

5. **Flatpak/Snap/AppImage** — If xrat is distributed through sandboxed formats,
   the `.desktop` file and tray integration need sandbox-specific conventions.
   Recommendation: out of scope; revisit if sandboxed packaging becomes a goal.

6. **`xrat doctor` command** — Should a diagnostics command exist to validate
   runtime binaries, database connectivity, config syntax, and daemon
   reachability? Useful for troubleshooting but separate from this phase.

---

## Out of Scope

- Windows system tray (supported by `tray-icon` but not a current target).
- Full minimize-to-tray for the TUI.
- Global hotkeys.
- Embedded web dashboard from tray.
- Drag-and-drop config import via tray.
- Flatpak/Snap/AppImage packaging.
- GUI configuration editor.

---

## Implementation Slices

### Slice A: Installation Documentation

1. Create `docs/src/01-getting-started/installation.md`.
2. Update `docs/src/SUMMARY.md`.
3. Update quickstart to begin with `xrat init`.
4. Update `docs/src/04-deployment/systemd.md`.

### Slice B: Shell Completion

1. Add `clap_complete` to `Cargo.toml`.
2. Add hidden `completions` command.
3. Implement completions handler writing to stdout.
4. Add CLI parser tests.

### Slice C: Release Workflow

1. Update `.github/workflows/release.yml` with pre-release validation.
2. Change artifact naming to target-triple format.
3. Add checksum generation.
4. Add completion and man page generation to the workflow.
5. Evaluate musl and ARM64 targets.

### Slice D: Desktop Entry and `xrat integrate`

1. Add `Integrate` to `src/cli/command.rs`.
2. Add `src/app/commands/integrate.rs`.
3. Implement `.desktop` file generation and icon asset installation from
   `media/icons/`.
4. Add CLI parser tests.

### Slice E: Documentation

1. Create/update all documentation pages listed in Section 6.
2. Verify mdBook build passes.

### Slice F: Notifications (Optional)

1. Add `notify-rust` as optional dependency with `notifications` feature.
2. Implement state transition detection and rate-limited notifications.

### Slice G: Tray Icon (Lowest Priority)

1. Add `tray-icon` as optional dependency with `tray` feature.
2. Add `Tray` to `src/cli/command.rs`.
3. Add `src/app/commands/tray.rs`.
4. Implement tray icon, context menu, daemon IPC polling, single instance.
5. Add CLI parser tests.

---

## Completion Criteria

This phase is complete when:

1. `xrat integrate` installs desktop entry and icon assets on Linux.
2. Shell completions are generated and included in release artifacts.
3. Man pages are generated and included in release artifacts.
4. Release workflow produces named, checksummed archives with completions and
   man pages.
5. Installation documentation covers prerequisites, install methods, and
   first-time setup.
6. `cargo fmt` and `cargo test -q` pass.
7. `xrat tray` shows a status icon with a working context menu (lowest
   priority).
8. The tray icon reflects daemon state via IPC.
9. `cargo build --features tray` succeeds on Linux.
