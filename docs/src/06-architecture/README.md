# Architecture

This section describes xrat's internal architecture, data flow, and module
structure for developers and contributors.

## Context Diagram

```mermaid
graph TB
    classDef user   fill:#3a2c1a,stroke:#dfa85b,color:#e6edf3
    classDef iface  fill:#1a2e1a,stroke:#5bdf8a,color:#e6edf3
    classDef core   fill:#1a2c3a,stroke:#5b8def,color:#e6edf3
    classDef engine fill:#1a3a1a,stroke:#5bdf5b,color:#e6edf3
    classDef store  fill:#1a3a3a,stroke:#5bdfd3,color:#e6edf3

    User(("User")):::user

    subgraph interfaces["User Interfaces"]
        CLI["CLI  (terminal)"]:::iface
        TUI["TUI  (ratatui)"]:::iface
        API["HTTP API  (axum)"]:::iface
    end

    subgraph xrat_core["xrat Core"]
        Daemon["Daemon Supervisor"]:::core
    end

    subgraph engines["Proxy Engines"]
        Xray["Xray-core"]:::engine
        SingBox["sing-box"]:::engine
    end

    DB[("SQLite / Postgres")]:::store

    User --> CLI
    User --> TUI
    User -- "HTTP" --> API

    CLI -- "IPC" --> Daemon
    TUI -- "IPC" --> Daemon

    Daemon -- "spawns" --> Xray
    Daemon -- "spawns" --> SingBox

    CLI --> DB
    Daemon --> DB
    API --> DB
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
