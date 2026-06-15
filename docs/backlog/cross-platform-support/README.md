# Cross-Platform Support Backlog

This folder tracks follow-up work needed to make xrat behave consistently
across Linux, macOS, FreeBSD, and OpenBSD. Windows is deferred and tracked
separately in `12-windows-support.md`.

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

Items 3–10 are code complete and compile clean on Linux but still need runtime
verification on real macOS/FreeBSD/OpenBSD hosts.

## Remaining

- `12-windows-support.md` — Windows track (reattach, daemon, desktop proxy,
  installer, release upgrades).
