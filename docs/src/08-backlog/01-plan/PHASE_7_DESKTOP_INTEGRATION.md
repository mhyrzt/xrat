# Phase 7: Desktop Integration (Desktop Entry + Tray Icon)

## Goal

Make xrat accessible outside the terminal by adding:

1. A **Freedesktop `.desktop` entry** so the TUI can be launched from app
   launchers (GNOME, KDE, macOS, etc.).
2. A **cross-platform tray icon** that shows connection status, quick actions
   (connect/disconnect), and optionally minimizes/maximizes the TUI window.

Target platforms: **Linux** (primary) and **macOS**.

## Why This Phase Exists

Phase 6 adds a full Ratatui TUI. However, the user must open a terminal and run
`xrat tui` manually each time. A `.desktop` entry makes xrat discoverable in the
system launcher. A tray icon provides persistent background presence, status
at-a-glance, and quick toggle without keeping the TUI open.

This is a common pattern for proxy/TUI tools:

- **Clash** and **Clash Verge** have system tray + desktop entries.
- **mihomo** (Clash Meta) provides tray support via `mihomo tray`.
- **nekoray** and **v2rayN** minimize to tray on Windows.

xrat should offer comparable desktop integration for Linux and macOS users.

## Desktop Entry

### Specification

A `.desktop` file following the
[Freedesktop Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry-spec/latest/):

```ini
[Desktop Entry]
Type=Application
Name=XRAT
Comment=Proxy/VPN configuration manager
Exec=xrat tui
Terminal=true
Categories=Network;
Keywords=proxy;vpn;xray;tui;
Icon=xrat-logo
```

Key decisions:

- **`Terminal=true`** — xrat is a TUI, so it must run inside a terminal
  emulator. Launcher compatibility varies (some launchers handle it, some
  silently fail). May need a wrapper that spawns a terminal emulator explicitly
  (e.g. `Exec=foot xrat tui` or `Exec=alacritty -e xrat tui`) if launcher
  support is inconsistent.
- **Icon** — the `.desktop` file references the icon name, which must be
  installed to a standard XDG icon path (see below).

### Installation Paths

| Path                                                  | Purpose                   |
| ----------------------------------------------------- | ------------------------- |
| `~/.local/share/applications/xrat.desktop`            | User-local desktop entry  |
| `~/.local/share/icons/hicolor/.../apps/xrat-logo.png` | User-local icon           |
| `/usr/share/applications/xrat.desktop`                | System-wide desktop entry |
| `/usr/share/icons/hicolor/.../apps/xrat-logo.png`     | System-wide icon          |

The icon should be installed at standard icon sizes: 48x48, 64x64, 128x128,
optionally 256x256. The source `media/xrat-logo.png` needs to be resized to
these sizes.

### macOS Dock Integration

macOS does not use `.desktop` files. An `.app` bundle (a directory with
`Contents/Info.plist` and `MacOS/` executable) is the equivalent. Since xrat is
a TUI, this would typically be a launcher script or a symlink in the bundle.

A minimal `.app` bundle wrapper:

```
XRAT.app/
  Contents/
    Info.plist
    MacOS/
      xrat-launcher  (shell script that opens Terminal.app and runs xrat tui)
    Resources/
      xrat-logo.icns (icon converted from PNG)
```

This is a **low-priority nicety**. The `.desktop` entry covers the Linux
workflow. macOS users can still launch from a terminal.

## Tray Icon

### Cross-Platform Approach

Use the [`tray-icon`](https://crates.io/crates/tray-icon) Rust crate. It wraps:

- **Linux**: `libappindicator` or `ayatana-appindicator` (GTK status icon via
  D-Bus). Falls back to `libayatana-appindicator` on modern distros.
- **macOS**: native NSStatusBar icon using Foundation/AppKit (via `objc2`).
- **Windows**: native `Shell_NotifyIcon` (if relevant later).

This gives a single API for status icon + menu across all three desktops.

### Required Crate

```toml
[dependencies]
tray-icon = "0.19"  # check latest version at decision time
```

### Architecture

The tray icon needs an **event loop** separate from the TUI. Two approaches:

**Option A: Standalone `xrat tray` command**

```
xrat tray --daemon
```

- Spawns a separate process with its own event loop.
- Communicates with the running xrat daemon (or runtime service) via IPC to get
  current status and send connect/disconnect commands.
- Icon updates based on daemon status (stopped → grey, running → green/colored).
- Menu items: Show TUI, Connect, Disconnect, Status, Quit.
- No TUI dependency in this process — small resource footprint.

**Option B: Built into `xrat tui`**

- The TUI event loop also drives a tray event loop in a background thread.
- Minimize-to-tray: pressing a key or closing the TUI hides instead of quitting.
- Re-show: tray "Show" menu item, or the background thread brings the terminal
  back.
- More complex: terminal reattachment after minimize is platform-specific and
  fragile. Ratatui's alternate screen does not survive backgrounding well.

**Recommendation: Option A** — standalone `xrat tray` command. It is simpler,
more reliable, and does not complicate the TUI lifecycle. The daemon IPC already
exists for the daemonization path.

### Tray Functionality

| Feature           | Details                                                                             |
| ----------------- | ----------------------------------------------------------------------------------- |
| **Status icon**   | Changes color/overlay based on runtime state (stopped, starting, running, error).   |
| **Context menu**  | Right-click: Show TUI, Connect/Disconnect, Status info, Quit.                       |
| **Notifications** | Optional: `notify-rust` to show connect/disconnect events via system notifications. |

### Tray Icon Assets

The `media/xrat-logo.png` is the source. Convert to:

- **Linux**: PNG icons at 22x22 (tray standard). Install to
  `~/.local/share/icons/hicolor/22x22/status/xrat-tray-{status}.png` (or use a
  single icon that tray-icon renders).
- **macOS**: 22x22 template image for NSStatusBar (template images support
  dark/light mode automatically on macOS).
- **Status variants**: Optionally generate overlay/tinted variants (e.g. grey
  for disconnected, green for connected, red for error). The `tray-icon` crate
  can switch the icon at runtime.

`tray-icon` handles icon loading from PNG bytes at runtime, so multiple variant
files or runtime tinting are both viable.

### Linux Dependencies

`tray-icon` requires a system-level status icon provider:

- **ayatana-appindicator** (modern, Ubuntu 20.10+, Debian, Fedora): package
  `libayatana-appindicator3-dev`.
- **libappindicator** (legacy): package `libappindicator3-dev`.

The Cargo crate will link against whichever is found. Document the required
system package in install docs. Feature-gate tray support so headless/builds
without the system lib can skip it.

### macOS Dependencies

No external system libraries needed. `tray-icon` links against system frameworks
via `objc2`. macOS builds work out of the box.

## Implementation Plan

### Slice 7.1: Desktop Entry and Icon Assets

Goal: install a `.desktop` file and resized icon assets.

Tasks:

- [ ] Add `install` or `post-install` script/CI step that: - copies
      `xrat.desktop` to `~/.local/share/applications/` - resizes
      `media/xrat-logo.png` to required sizes and copies to
      `~/.local/share/icons/hicolor/` - runs `update-desktop-database` or
      `gtk-update-icon-cache` if available
- [ ] Generate macOS `.icns` and `.app` bundle as optional extras.
- [ ] Add `xrat integrate` command that auto-installs the desktop entry and icon
      assets (user-scoped, no sudo needed).
- [ ] Optionally add a `--install-desktop-entry` flag to `xrat tui`.
- [ ] Document the desktop entry in install docs.

### Slice 7.2: Tray Icon Crate and System Deps

Goal: add `tray-icon` dependency and document system requirements.

Tasks:

- [ ] Add `tray-icon` to `Cargo.toml`.
- [ ] Feature-gate tray icon behind a `tray` feature (default off).
- [ ] Add build-time detection or doc guidance for
      `libayatana-appindicator3-dev` on Linux.
- [ ] Ensure macOS build succeeds without extra system deps.

### Slice 7.3: `xrat tray` Command

Goal: launch a tray icon process.

Tasks:

- [ ] Add `xrat tray` CLI command in `src/cli/` with optional `--daemon` flag.
- [ ] Add `src/app/commands/tray.rs` handler.
- [ ] Initialize `tray-icon` with the app icon and a default "disconnected"
      status.
- [ ] Build a context menu with: - Show TUI (spawns `xrat tui` in a terminal) -
      Toggle status (connect/disconnect) - Status label (read-only) - Quit
- [ ] Connect to daemon IPC to query current runtime state.
- [ ] Update tray icon based on daemon state changes (polling or IPC event
      stream).

### Slice 7.4: Tray ↔ Daemon IPC

Goal: tray process communicates with the daemon for status and control.

Tasks:

- [ ] Define IPC messages needed: `GetStatus`, `Connect`, `Disconnect`, `Quit`.
- [ ] Reuse or extend the existing daemon IPC protocol.
- [ ] Tray process sends commands and receives state responses.
- [ ] Handle daemon-not-running gracefully (show "daemon not running" state).

### Slice 7.5: Notifications (Optional)

Goal: show system notifications for important events.

Tasks:

- [ ] Add `notify-rust` behind a feature flag.
- [ ] Show notification on connect/disconnect/error.
- [ ] Gate notifications to avoid spam.

### Slice 7.6: Documentation

Tasks:

- [ ] Document `xrat tray` command and its dependencies.
- [ ] Document system requirements (Linux: `libayatana-appindicator3-dev`,
      `libnotify`).
- [ ] Document desktop entry installation.
- [ ] Add platform-specific notes (Wayland vs X11 for tray icons, macOS .app
      bundle).
- [ ] Update `docs/src/SUMMARY.md`.

## Open Questions

1. **Wayland compatibility** — `libappindicator`/`ayatana-appindicator` has
   inconsistent Wayland support. On GNOME Wayland, tray icons require the
   `AppIndicator` extension or KStatusNotifierItem protocol. Should the tray
   feature document this caveat or implement a D-Bus status notifier item
   directly?

2. **TUI minimize-to-tray** — Should `xrat tui` support hiding to tray (instead
   of quitting) when `xrat tray` is running? This requires terminal reattachment
   which is fragile. Delaying this to a later polish slice.

3. **Single tray instance** — How to prevent multiple `xrat tray` processes? A
   PID file or a lockfile in `$XDG_RUNTIME_DIR/xrat-tray.pid`.

4. **Auto-start** — Should `xrat tray` support `--autostart` to install a
   desktop autostart entry (`~/.config/autostart/xrat-tray.desktop`)? Common
   pattern but adds scope.

5. **macOS .app bundle** — Is a Terminal.app-launcher bundle worth maintaining,
   or should we rely on the user launching from their shell?

6. **Tray icon variants** — Should status be communicated by changing icon color
   (requires 3+ PNG assets) or by adding a small overlay badge (requires runtime
   compositing)?

7. **Flatpak/Snap/AppImage** — If xrat is distributed through these formats, the
   `.desktop` file and tray integration need to follow the sandbox conventions.
   Out of scope for now but worth noting.

## Completion Criteria

Phase 7 can be considered complete when:

1. `xrat integrate` installs a working `.desktop` entry and icon assets on
   Linux.
2. xrat appears in the system app launcher after installation.
3. `xrat tray` starts a system tray icon with a context menu.
4. The tray icon reflects the current daemon/runtime status.
5. The tray menu can connect/disconnect the proxy via daemon IPC.
6. `cargo build` succeeds with the `tray` feature on both Linux and macOS.
7. System dependencies are documented for Linux.
8. `cargo fmt` and `cargo test -q` pass.

## Out of Scope

- Windows system tray (not relevant now, but `tray-icon` supports it trivially
  if needed later).
- Full minimize-to-tray for the TUI (see Open Questions above).
- Global hotkeys.
- Embedded web dashboard from tray.
- Drag-and-drop config import via tray.
