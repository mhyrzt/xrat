---
id: m-0
title: "Windows Support"
---

## Description

Legacy path: `docs/backlog/feature/cross-platform-support/README.md`

# Cross-Platform Support Backlog

This folder tracks follow-up work needed to make xrat behave consistently
across Linux, macOS, FreeBSD, OpenBSD, and Windows. Windows is still deferred
from the supported platform matrix and is tracked as a focused workstream.

## Done

Completed items have had their task files removed; see git history for the
detail. Implemented so far:

1. ICMP ping flags per BSD variant.
2. Shell detection via parent process (not `/proc`).
3. Process reattach via `sysinfo` (works on macOS/BSD/Windows).
4. Daemon install: launchd user agents on macOS (`packaging/launchd/`).
5. Daemon install: rc.d scripts on FreeBSD/OpenBSD (`packaging/rc.d/`).
6. Release upgrades for macOS (darwin triples + CI darwin archives).
7. Desktop proxy on macOS via `networksetup`.
8. `install.sh` accepts macOS.
9. BSD clipboard confirmed (arboard X11 backend, no change needed).
10. `install.sh` portable Unix paths and checksum tools.
11. User docs scoped per-OS + platform support matrix.

Items 3-10 are code complete and compile clean on Linux but still need runtime
verification on real macOS/FreeBSD/OpenBSD hosts.

## Remaining

- `12-windows-support.md` - Windows support overview.
- `12-01-windows-ipc-and-daemon-runtime.md` - Windows daemon IPC and runtime
  control.
- `12-02-windows-service-install.md` - Windows service install/uninstall.
- `12-03-windows-desktop-proxy.md` - Windows desktop proxy integration.
- `12-04-windows-installer.md` - PowerShell installer.
- `12-05-windows-release-and-upgrade.md` - release archives and self-upgrade.
- `12-06-windows-runtime-verification.md` - Windows runtime acceptance checks.


---

Legacy path: `docs/backlog/feature/cross-platform-support/12-windows-support.md`

# 12. Windows support

**Difficulty:** Large, P3 - spans daemon IPC, service management, desktop
proxy integration, installer/release packaging, and runtime verification.

xrat currently compiles for Windows, but Windows is not a supported runtime
target. Several platform integrations either use Unix-only code paths or return
`UnsupportedPlatform`. Promote Windows only after the focused tasks below are
implemented and verified on a Windows host or `windows-latest` CI runner.

## Task set

1. `12-01-windows-ipc-and-daemon-runtime.md` - add a Windows daemon transport
   so CLI commands can control `xrat daemon run-server`.
2. `12-02-windows-service-install.md` - install and uninstall the daemon as a
   Windows Service.
3. `12-03-windows-desktop-proxy.md` - support `xrat proxy desktop` through the
   per-user WinINET proxy settings.
4. `12-04-windows-installer.md` - add a PowerShell installer for release
   archives.
5. `12-05-windows-release-and-upgrade.md` - publish Windows archives and make
   `xrat upgrade` handle Windows safely.
6. `12-06-windows-runtime-verification.md` - verify reattach, clipboard, engine
   spawning, setup behavior, and end-to-end workflows on Windows.

## Current blockers

- Daemon IPC is Unix-socket based. `src/app/daemon/ipc/client/mod.rs` selects
  `unsupported_impl` on non-Unix, and the server path in
  `src/app/daemon/ipc/handler/mod.rs` only binds `UnixListener`.
- `src/app/commands/daemon_install.rs` falls through to the unsupported stub
  outside Linux, macOS, FreeBSD, and OpenBSD.
- `src/app/commands/proxy/desktop.rs` only has Linux and macOS implementations.
- `src/app/commands/upgrade/release.rs` assumes `.tar.gz`, `tar`, and
  `sha256sum`; Windows release archives should be `.zip`.
- `.github/workflows/release.yml` does not build or publish Windows artifacts.

## Completion criteria

- Windows has a documented supported behavior in the user docs and platform
  matrix.
- `cargo test --locked` passes on `windows-latest`.
- A Windows runtime smoke pass covers install, setup, daemon control, service
  install/uninstall, desktop proxy enable/status/disable, release install, and
  self-upgrade.


---
