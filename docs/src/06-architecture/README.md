# Architecture

This section describes xrat's internal architecture, data flow, and module
structure for developers and contributors.

## Context Diagram

```mermaid
graph TB
    User((User))
    CLI[CLI / Terminal]
    TUI[TUI ratatui]
    Daemon[Daemon Supervisor]
    API[HTTP API axum]
    Xray[Xray-core]
    SingBox[sing-box]
    DB[(SQLite/Postgres)]

    User --> CLI
    User --> TUI
    User -- http --> API
    CLI -- ipc --> Daemon
    TUI -- ipc --> Daemon
    Daemon -- spawns --> Xray
    Daemon -- spawns --> SingBox
    CLI --> DB
    Daemon --> DB
    API --> DB

    style Xray fill:#4a9eff
    style SingBox fill:#4a9eff
    style Daemon fill:#ff9a4a
    style DB fill:#4aff9a
```

## Pages

| Page                                                  | Description                                             |
| ----------------------------------------------------- | ------------------------------------------------------- |
| [Module Structure](module-structure.md)               | Source tree, module responsibilities, dependency graph  |
| [Config Generation](config-generation.md)             | How engine JSON configs are generated from nodes        |
| [Import Pipeline](import-pipeline.md)                 | End-to-end subscription import flow                     |
| [Daemon Architecture](daemon-architecture.md)         | Daemon process, IPC protocol, supervisor event loop     |
| [Runtime Lifecycle](runtime-lifecycle.md)             | Session state machine, connect/replace/disconnect flows |
| [Test Pipeline](test-pipeline.md)                     | Probe execution, test stages, output formatting         |
| [Database Schema](../05-reference/database-schema.md) | Full SQL DDL and per-table column reference             |
