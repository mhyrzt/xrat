---
id: TASK-4
title: 12.01 Windows IPC and daemon runtime control
status: To Do
assignee: []
created_date: '2026-07-05 14:43'
updated_date: '2026-07-05 14:44'
labels:
  - legacy-import
  - feature
  - cross-platform
milestone: m-0
dependencies: []
priority: medium
ordinal: 1201
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/feature/cross-platform-support/12-01-windows-ipc-and-daemon-runtime.md`

# 12.01 Windows IPC and daemon runtime control

**Difficulty:** Large, P3.

Daemon support on Windows must start with IPC. Installing a Windows Service is
not useful until CLI commands such as `xrat daemon status`, `xrat connect`, and
`xrat rotate status` can talk to the daemon process.

## Current state

- `src/app/commands/daemon.rs` computes a `socket_path` and uses the shared
  `ipc::*_daemon` helpers for all control commands.
- `src/app/daemon/ipc/client/mod.rs` uses `unix_impl` on Unix and
  `unsupported_impl` on non-Unix.
- `src/app/daemon/ipc/handler/mod.rs` serves a `tokio::net::UnixListener` on
  Unix and returns `UnsupportedPlatform` elsewhere.
- The wire payloads, request kinds, response types, and supervisor dispatch are
  transport-agnostic and should be reused.

## Target behavior

- Windows daemon commands use a local, single-user IPC endpoint.
- The daemon rejects startup when another reachable daemon endpoint already
  exists.
- Existing command semantics are preserved: start/restart/status/stop,
  connect/disconnect, runtime replace, and rotation control all use the same
  request and response payloads as Unix.
- User-facing output should say `endpoint` or another neutral label on Windows,
  instead of exposing a Unix `socket` path.

## Implementation notes

- Prefer a Windows named pipe transport compatible with Tokio async I/O. A
  local loopback TCP fallback is acceptable only if named pipes add too much
  dependency or security complexity.
- Introduce a small transport abstraction for client connect, server accept,
  endpoint display, readiness checks, and stale endpoint cleanup.
- Keep Unix socket behavior unchanged.
- Do not change the daemon protocol JSON shape unless the transport requires a
  versioned extension.
- Place Windows-only endpoint naming under the existing runtime directory
  context or derive a stable per-user pipe name from it.

## Tests and verification

- Add transport-level unit tests for endpoint naming and display behavior.
- Add Windows integration tests for ping, status, shutdown, runtime replace,
  and malformed protocol requests.
- Keep existing Unix IPC tests passing unchanged.
- Verify `xrat daemon start`, `status`, `restart`, and `stop` on Windows with
  no service installed.

## Completion criteria

- Non-service daemon control works on Windows.
- `unsupported_impl` is no longer selected for Windows.
- Windows CI runs IPC tests instead of skipping the daemon client/server path.
<!-- SECTION:DESCRIPTION:END -->
