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
