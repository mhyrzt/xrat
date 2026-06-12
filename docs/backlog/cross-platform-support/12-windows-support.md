# 12.12 Large, P3: Windows support

**Difficulty:** Large — spans several subsystems; treat as its own track.

xrat compiles for Windows but several platform integrations silently no-op or
return errors there. This file tracks the gaps so Windows can be promoted from
"compiles" to "supported" later. None of this is implemented yet.

## Gaps

### Process reattach
**File:** `src/app/runtime_service/reattach/process.rs`

The reattach inspector now uses `sysinfo`, which works on Windows, so this is
likely already functional. Confirm `Process::exe()` and `Process::cmd()` return
useful values for the spawned xray/sing-box engine on Windows, and that
`xray_runtime::process_is_running` behaves. Add a Windows runtime check.

### Daemon install
**File:** `src/app/commands/daemon_install.rs`

`install`/`uninstall` fall through to the `#[cfg(not(any(linux, macos, freebsd,
openbsd)))]` stub returning `UnsupportedPlatform`. A Windows path needs either a
Windows Service (via `sc.exe` / SCM) or a Scheduled Task. A service is the
better fit for `daemon run-server`. New `packaging/windows/` assets and a
`#[cfg(windows)]` install/uninstall using `sc create` / `sc delete` (or the
`windows-service` crate) are required.

### Desktop proxy
**File:** `src/app/commands/proxy/desktop.rs`

`run` returns `UnsupportedPlatform` on Windows. Implement a `#[cfg(windows)]`
path setting the WinINET proxy via the registry
(`HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings`:
`ProxyEnable`, `ProxyServer`, `AutoConfigURL`) plus an `InternetSetOption`
refresh, or shell out to `netsh winhttp set proxy`.

### Installer
**File:** `install.sh` (POSIX only)

`install.sh` is bash and rejects non-Unix. Windows needs a separate
`install.ps1` (PowerShell): detect arch, download the
`*-pc-windows-msvc` release archive, verify the checksum
(`Get-FileHash -Algorithm SHA256`), unpack, place `xrat.exe` on `PATH`, and
optionally register the service.

### Release upgrades
**File:** `src/app/commands/upgrade/release.rs`, `.github/workflows/release.yml`

`detect_arch()` has no Windows arm. Add `("windows", "x86_64") =>
"x86_64-pc-windows-msvc"` (and `aarch64`) once the release workflow builds and
uploads Windows archives (add a `windows-latest` matrix entry; package a `.zip`
rather than `.tar.gz`, and adjust `install_binary` for the
running-executable-replacement constraint on Windows — a running `.exe` cannot
be overwritten in place, so stage-and-rename-on-restart is needed).

### Clipboard
`arboard` already has a Windows backend (`platform/windows.rs`), so the TUI
clipboard should work. Confirm.

## Verification
Requires a Windows host or CI runner. Out of scope until the items above are
scheduled.
