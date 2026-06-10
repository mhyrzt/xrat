# Cross-Platform Support Backlog

This folder tracks follow-up work needed to make xrat behave consistently
across Linux, macOS, FreeBSD, OpenBSD, and Windows where applicable.

Items here focus on platform-specific assumptions in CLI/runtime behavior,
installer scripts, daemon integration, process inspection, desktop integration,
and user documentation. Each numbered file describes one scoped task with its
expected implementation path and verification notes.

## Already Done

- `1-ping-flags.md` — Fixed ICMP probing so FreeBSD uses `ping -t` and OpenBSD
  uses `ping -w`, while preserving Linux, macOS, Windows, and fallback behavior.
- `2-shell-detection.md` — Replaced Linux-only `/proc/{ppid}/comm` shell
  detection with best-effort parent process detection via `sysinfo`.
