# Cross-Platform Support Backlog

This folder tracks follow-up work needed to make xrat behave consistently
across Linux, macOS, FreeBSD, and OpenBSD. Windows is deferred and tracked
separately in `12-windows-support.md`.

Items here focus on platform-specific assumptions in CLI/runtime behavior,
installer scripts, daemon integration, process inspection, desktop integration,
and user documentation. Each numbered file describes one scoped task with its
expected implementation path and verification notes.

## Already Done

- `1-ping-flags.md` — Fixed ICMP probing so FreeBSD uses `ping -t` and OpenBSD
  uses `ping -w`, while preserving Linux, macOS, Windows, and fallback behavior.
- `2-shell-detection.md` — Replaced Linux-only `/proc/{ppid}/comm` shell
  detection with best-effort parent process detection via `sysinfo`.

## Implemented (code complete; runtime verification on macOS/BSD pending)

These landed build-gated and compile clean on Linux; they still need to be
exercised on real macOS/FreeBSD/OpenBSD hosts.

- `3-process-reattach.md` — Reattach inspector uses `sysinfo` instead of
  `/proc`, so it works on macOS/BSD (and Windows).
- `4-daemon-install-macos.md` — `daemon install`/`uninstall` support launchd
  user agents on macOS (`packaging/launchd/`).
- `5-daemon-install-bsd.md` — `daemon install`/`uninstall` support rc.d scripts
  on FreeBSD/OpenBSD (`packaging/rc.d/`).
- `6-release-upgrades-macos.md` — `detect_arch()` returns darwin triples and the
  release workflow builds darwin archives.
- `7-desktop-proxy-macos.md` — `proxy desktop` supports `networksetup` on macOS
  alongside the existing GNOME path.
- `8-install-sh-macos.md` + `10-install-sh-portable-unix.md` — `install.sh`
  accepts macOS, uses target triples, picks `sha256sum`/`shasum`, and gates
  systemd/loginctl/desktop steps to Linux.
- `9-clipboard-bsd.md` — Confirmed `arboard`'s unix (non-mac) backend covers
  FreeBSD/OpenBSD via X11 with no change needed.

## Remaining

- `11-unix-platform-docs.md` — Update user docs and add the support matrix once
  the above is verified on hardware.
- `12-windows-support.md` — Windows track (reattach, daemon, desktop proxy,
  installer, release upgrades).
