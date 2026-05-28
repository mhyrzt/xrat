# Documentation Plan

## Goal

Create user-facing and developer-facing documentation for xrat under `docs/src/`,
focused on xrat itself (not Xray/V2Ray/sing-box internals).

## Current Structure

```
docs/src/
├── README.md                          (home / project overview)
├── SUMMARY.md                         (mdBook navigation)
│
├── 01-getting-started/
│   ├── README.md                      (installation, building from source)
│   ├── quickstart.md                  (import → test → connect flow)
│   └── configuration.md              (config.toml reference — all sections)
│
├── 02-cli/
│   ├── README.md                      (global flags, command overview)
│   ├── import.md                      (import + add)
│   ├── list.md                        (list configs + list subscriptions)
│   ├── parse.md                       (parse without persisting)
│   ├── test.md                        (test command — all flags, stages, output)
│   ├── scan.md                        (IP scanner)
│   ├── runtime.md                     (connect, disconnect, status)
│   ├── daemon.md                      (daemon start/status/stop)
│   ├── proxy.md                       (proxy start/status/rotate/stop)
│   └── serve.md                       (HTTP API server)
│
├── 03-features/
│   ├── README.md                      (feature overview)
│   ├── importing.md                   (subscription URLs, files, raw text, base64)
│   ├── testing.md                     (5-stage probe pipeline, failure classification)
│   ├── runtime-management.md          (connect lifecycle, session state, reattach)
│   ├── daemon-and-ipc.md             (supervisor, Unix socket IPC protocol)
│   ├── auto-rotation.md              (triggers, cooldown, candidate selection)
│   ├── ip-scanning.md                (TCP scan, persistence, history)
│   ├── http-api.md                   (routes, auth, response shapes)
│   └── deduplication.md              (dedup key format, versioning)
│
├── 04-deployment/
│   ├── README.md
│   ├── systemd.md                    (user service examples)
│   └── database-backends.md          (SQLite vs Postgres setup)
│
├── 05-reference/
│   ├── README.md
│   ├── protocols.md                  (supported protocols table, URI schemes, engine routing)
│   ├── config-file.md                (config.toml full reference)
│   ├── database-schema.md            (tables, columns, migrations)
│   └── error-codes.md                (AppError variants, FailureKind categories)
│
├── 06-architecture/
│   ├── README.md                     (module map, data flow)
│   ├── config-generation.md          (Node → Xray/sing-box JSON pipeline)
│   └── module-structure.md           (src/ tree, responsibilities)
│
├── 07-config/                        (existing — Xray JSON internals, renumbered from 01)
│   └── ...
│
└── 08-backlog/                       (existing — planning/validation, renumbered from 02)
    └── ...
```

## Key Decisions

1. **Renumber existing sections** — `01-config/` → `07-config/`, `02-backlog/` → `08-backlog/`
2. **`01-getting-started/`** — onboarding: install, quickstart, config.toml overview
3. **`02-cli/`** — one page per command group, all flags with examples
4. **`03-features/`** — conceptual deep-dives into each major subsystem
5. **`04-deployment/`** — operational concerns (systemd, DB backends)
6. **`05-reference/`** — lookup material (protocols, schema, errors)
7. **`06-architecture/`** — developer-oriented module map and config generation pipeline

## What's NOT Included

- Xray/V2Ray/sing-box config format documentation (already in `07-config/`)
- Phase plans and parity checklists (stays in `08-backlog/`)
- TUI docs (not implemented yet — add in Phase 6)

## Checklist

### 01-getting-started/
- [x] `README.md` — installation, building from source
- [x] `quickstart.md` — import → test → connect flow
- [x] `configuration.md` — config.toml reference overview

### 02-cli/
- [x] `README.md` — global flags, command overview
- [x] `import.md` — import + add
- [x] `list.md` — list configs + list subscriptions
- [x] `parse.md` — parse without persisting
- [x] `test.md` — test command, all flags, stages, output formats
- [x] `scan.md` — IP scanner
- [x] `runtime.md` — connect, disconnect, status
- [x] `daemon.md` — daemon start/status/stop
- [x] `proxy.md` — proxy start/status/rotate/stop
- [x] `serve.md` — HTTP API server

### 03-features/
- [x] `README.md` — feature overview
- [x] `importing.md` — subscription URLs, files, raw text, base64 decoding
- [x] `testing.md` — 5-stage probe pipeline, failure classification
- [x] `runtime-management.md` — connect lifecycle, session state, reattach
- [x] `daemon-and-ipc.md` — supervisor, Unix socket IPC protocol
- [x] `auto-rotation.md` — triggers, cooldown, candidate selection
- [x] `ip-scanning.md` — TCP scan, persistence, history
- [x] `http-api.md` — routes, auth, response shapes
- [x] `deduplication.md` — dedup key format, versioning

### 04-deployment/
- [x] `README.md`
- [x] `systemd.md` — user service examples
- [x] `database-backends.md` — SQLite vs Postgres setup

### 05-reference/
- [x] `README.md`
- [x] `protocols.md` — supported protocols, URI schemes, engine routing
- [x] `config-file.md` — config.toml full reference
- [x] `database-schema.md` — tables, columns, migrations
- [x] `error-codes.md` — AppError variants, FailureKind categories

### 06-architecture/
- [x] `README.md` — module map, data flow
- [x] `config-generation.md` — Node → Xray/sing-box JSON pipeline
- [x] `module-structure.md` — src/ tree, responsibilities

### Housekeeping
- [x] Rename `01-config/` → `07-config/`
- [x] Rename `02-backlog/` → `08-backlog/`
- [x] Update `SUMMARY.md` with new structure
- [x] Update internal links across all pages
