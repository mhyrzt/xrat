# 11.11 Easy, P2: Document Unix platform support matrix

**Difficulty:** Easy — half day.

**Files:** `docs/src/`, especially installation, daemon, proxy, and runtime docs.

After adding macOS/BSD support, update user-facing docs so Linux-only behavior is
not presented as universal.

Known docs that need revision:

- `docs/src/02-cli/proxy.md` describes shell detection via `/proc/$PPID/comm`.
  Update after item 02 lands.
- `docs/src/02-cli/proxy.md` says desktop proxy is Linux-only. Split supported
  backends by OS: GNOME/gsettings on Linux, networksetup on macOS once item 07
  lands, unsupported on BSD unless implemented.
- `docs/src/02-cli/daemon.md` and `docs/src/04-deployment/systemd.md` describe
  systemd as the daemon install path. Add launchd and rc.d pages/sections when
  items 04 and 05 land.
- `docs/src/03-features/runtime-management.md` and
  `docs/src/03-features/daemon-and-ipc.md` mention `/proc/<pid>/cmdline` for
  reattach. Update after item 03 lands.
- `docs/src/01-getting-started/installation.md` and manual binary install docs
  list only Linux musl release archives. Add macOS archives after item 06.

Add a concise support matrix with rows like:

| Feature | Linux | macOS | FreeBSD | OpenBSD |
| --- | --- | --- | --- | --- |
| CLI/config/import/list | yes | yes | expected | expected |
| daemon runtime IPC | Unix socket | Unix socket | Unix socket | Unix socket |
| daemon install | systemd user | launchd | rc.d | rc.d/manual |
| runtime reattach | sysinfo/proc | sysinfo/libproc | sysinfo/sysctl | sysinfo/ps fallback |
| desktop proxy | GNOME/gsettings | networksetup | unsupported | unsupported |
| release upgrade | musl tarball | darwin tarball | source/manual | source/manual |

**Verification:** `mdbook build` and grep docs for stale Linux-only, systemd-only,
and `/proc` language that should be scoped.
