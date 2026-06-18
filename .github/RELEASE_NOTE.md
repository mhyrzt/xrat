## xrat v0.6.0

Cross-platform support: xrat now installs and runs on macOS and BSD in addition
to Linux.

### Platform support

- **macOS**: full first-class support — Apple Silicon and Intel release archives
  (`aarch64-apple-darwin`, `x86_64-apple-darwin`), `launchd` user agents for the
  daemon and API services, and desktop proxy control via `networksetup` (web,
  secure, SOCKS, and PAC).
- **FreeBSD/OpenBSD**: daemon install via `rc.d` scripts (enable/start requires
  root).
- **Linux**: unchanged — `systemd` user services, GNOME/`gsettings` desktop
  proxy, and `.desktop`/icon assets continue to ship as before.

### Features

- **daemon**: `daemon install`/`uninstall` dispatch per-OS — `systemd` on Linux,
  `launchd` on macOS, `rc.d` on FreeBSD/OpenBSD.
- **proxy**: `proxy desktop` adds a macOS `networksetup` backend alongside the
  existing Linux GNOME path (enable/disable/status/toggle).
- **upgrade**: `detect_arch()` now resolves `apple-darwin` triples on macOS, and
  the release workflow builds darwin archives on `macos-latest`.
- **installer**: `install.sh` detects the release target triple per OS/arch,
  verifies with `sha256sum` or `shasum`, and gates `systemd`/`loginctl`/desktop
  steps to Linux so macOS installs cleanly.

### Fixes

- **reattach**: process inspection now uses `sysinfo` instead of reading
  `/proc/{pid}/exe` and `/proc/{pid}/cmdline`, which do not exist on macOS/BSD.
  Runtime reattach previously failed silently on those targets despite being
  `cfg(unix)`.

### Upgrade notes

- No new database migrations in this release.
- Linux behavior and on-disk layout are unchanged; no action needed when
  upgrading an existing Linux install.
- macOS desktop archives intentionally omit the Linux/XDG `.desktop` entry and
  hicolor icons.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.5.3...v0.6.0
