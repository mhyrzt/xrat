---
id: TASK-17
title: Split AppError Into Layered Error Types
status: To Do
assignee: []
created_date: '2026-07-05 14:43'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-2
dependencies: []
priority: medium
ordinal: 23
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/1-foundation/23-split-apperror-by-layer.md`

# Split AppError Into Layered Error Types

## Finding

### [Priority: High] Split the AppError god-enum by layer

**Files involved:**

- `src/app/error.rs`
- `src/app/commands/` (all handlers that return `app::Result`)
- `src/xray/process/errors.rs`
- `src/singbox/process_mgmt.rs`
- `src/config/error.rs`
- `src/db/error.rs`
- `src/server/error.rs`

**Problem:** `src/app/error.rs` defines a single `AppError` enum with 26 variants
that mixes three unrelated concerns:

- Infrastructure library errors imported via `#[from]`: `std::io::Error`,
  `toml::de::Error`, `reqwest::Error`, `serde_json::Error`, `DbError`,
  `DecodeError`, `tokio::task::JoinError`.
- Pure domain/application rules: `NoSupportedConfig`, `MultipleConfigsForAdd`,
  `RawJsonImportUnsupported`, `RuntimeSessionAlreadyActive`,
  `NoRuntimeInboundEnabled`, `UnsupportedProtocol`, `InvalidArgument`.
- Engine/process internals: `XraySpawn`, `XrayExited`, `XrayStartupTimeout`,
  `SingboxSpawn`, `SingboxExited`, `SingboxStartupTimeout`, `GeoipDownload`.

Because `#[from] reqwest::Error` and friends live at the top of the application
error type, every module that returns `app::Result` transitively couples to
`reqwest`, `sqlx`, `toml`, and `serde_json`, even when it never performs HTTP,
SQL, or TOML work.

**Why this change is needed:** This directly blocks the port work in `14-http-
client-port` and `15-process-spawner-port`. A port cannot own its own error type
(`HttpError`, `ProcessError`) if the application's top-level enum already
`#[from]`s the concrete library that the port is meant to hide. The leaky
`#[from] reqwest::Error` is the reason HTTP cannot be faked: any layer can
construct an `AppError::Http` directly. The mixed enum also makes exhaustive
matching at adapter boundaries (CLI exit codes, HTTP status codes in
`server/error.rs`) coarse and error-prone, and it makes the domain rules harder
to unit-test in isolation because they share a type with I/O failures.

**How to implement it:** Introduce per-layer error types that already partly
exist (`config/error.rs`, `db/error.rs`, `xray/process/errors.rs`,
`server/error.rs`) and converge on a thin top-level enum:

- Keep domain/application rule variants in `AppError` (or a new
  `app::DomainError`).
- Move engine variants into the xray/singbox layers and convert at the boundary
  with `#[from]` on a narrow `EngineError`, not on raw process internals.
- Replace top-level `#[from] reqwest::Error` with conversion from a port-owned
  `HttpError` (introduced by `14-http-client-port`); same for any future
  `ProcessError` from `15-process-spawner-port`.
- Map each layer error into the adapter response once, at the edge (CLI message,
  HTTP status, daemon IPC payload).

**Positive effect on the codebase:** Removes hidden coupling to `reqwest`,
`sqlx`, and `toml` from modules that do no such I/O. Unblocks port error
ownership for HTTP and process work. Lets adapters map errors to exit codes and
HTTP statuses by matching on a small, stable domain enum instead of a 26-variant
grab bag. Domain rules become testable without constructing infra errors.

**Suggested target architecture:** Each layer (db, config, xray, singbox, http
port, process port) owns a focused error type; the application layer holds only
domain-rule variants plus `#[from]` conversions from those layer errors; adapters
translate the layer/application error into transport-specific responses at the
boundary.

**Risk / migration notes:** Medium risk because error types thread through every
handler. Do it incrementally: first move the engine variants behind an
`EngineError`, then replace the `reqwest` `#[from]` as part of `14-http-client-
port`, then split domain rules last. Keep `app::Result` as the public alias
throughout so call sites stay stable during migration. This item should land
alongside or just before `14`/`15`.
<!-- SECTION:DESCRIPTION:END -->
