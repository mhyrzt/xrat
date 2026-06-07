# Daemon Architecture

The daemon is a background process that owns the managed Xray runtime, accepts
IPC requests from CLI/TUI clients, runs health checks, and drives auto-rotation.

## Process Model

```mermaid
graph TB
    classDef cli    fill:#1a2744,stroke:#4a9eff,color:#e6edf3
    classDef proc   fill:#2a1a3a,stroke:#b070df,color:#e6edf3
    classDef client fill:#1a2e1a,stroke:#5bdf8a,color:#e6edf3
    classDef sock   fill:#2e2a1a,stroke:#dfba5b,color:#e6edf3

    CLI["xrat daemon start"]:::cli
    PARENT["Parent process<br/>validates config, forks child"]:::proc
    CHILD["Child process<br/>execs 'xrat daemon run-server'"]:::proc
    SOCK["Unix socket<br/>/path/to/xrat.sock"]:::sock
    CLIENT1["xrat status"]:::client
    CLIENT2["xrat connect"]:::client
    CLIENT3["xrat rotate enable"]:::client

    CLI --> PARENT
    PARENT -- "std::process::Command" --> CHILD
    CHILD -- "creates" --> SOCK
    CLIENT1 -- "IPC request" --> SOCK
    CLIENT2 -- "IPC request" --> SOCK
    CLIENT3 -- "IPC request" --> SOCK
```

## Daemon Startup Sequence

```mermaid
sequenceDiagram
    participant User as User
    participant CLI as xrat daemon start
    participant Parent as Parent Process
    participant Child as Child (run-server)
    participant Sock as Unix Socket
    participant DB as Database

    User->>CLI: xrat daemon start
    CLI->>Parent: validate ports, clean old socket
    Parent->>Child: fork + exec (XRAT_DAEMON_PARENT_PID)
    Child->>Child: init SupervisorState
    Child->>DB: reattach stale sessions
    Child->>Sock: listen on Unix socket
    Child->>Parent: IPC Ping response
    Parent->>User: "Daemon started (pid: N)"
    Note over Child: Enter event loop
```

## Supervisor Event Loop

The supervisor runs a `tokio::select!` loop with three concurrent branches (IPC
accept, health-check tick, rotation tick). The structure of the loop itself is
internal — what matters is the set of `SupervisorEvent` messages the loop
handles and the resulting `SupervisorState` mutations.

```mermaid
stateDiagram-v2
    [*] --> SupervisorRunning: daemon run-server

    state SupervisorRunning {
        [*] --> AcceptIPC: socket listener
        [*] --> HealthCheck: interval tick
        [*] --> RotationTick: interval tick

        AcceptIPC --> HandleIPC: connection received
        HandleIPC --> AcceptIPC: response sent

        HealthCheck --> CheckEndpoints: tick
        CheckEndpoints --> HealthCheck: done

        RotationTick --> EvalRotation: tick
        EvalRotation --> RotationTick: no trigger
        EvalRotation --> DoRotation: trigger active
        DoRotation --> RotationTick: done
    }

    SupervisorRunning --> SupervisorStopped: DaemonShutdown IPC
    SupervisorStopped --> [*]: exit
```

## IPC Protocol

The daemon communicates with CLI clients over a Unix socket using newline-
delimited JSON. The wire envelope is `DaemonRequest` / `DaemonResponse<T>` (see
`app/daemon/ipc/types.rs`); the actual operations are variants on
`DaemonRequestKind`.

### Request Envelope

```rust
pub struct DaemonRequest {
    pub protocol_version: u16,
    pub request: DaemonRequestKind,
}

pub enum DaemonRequestKind {
    DaemonPing,
    DaemonShutdown,
    RuntimeStatus,
    RuntimeConnect { config_id: i64 },
    RuntimeReplace {
        trigger: RotationTrigger,
        candidate_id: Option<i64>,
    },
    RuntimeDisconnect,
    ProxyStart,
    ProxyStatus,
    ProxyStop,
}
```

`RuntimeConnect` and `RuntimeReplace` carry the operation inputs inline; the
other variants are unit-only. `DaemonResponse<T>` is generic, wraps a
`DaemonResponseCode` (`Ok | Busy | NotFound | InvalidState | InternalError`),
and carries a typed payload (`PingPayload`, `RuntimeStatusPayload`,
`RuntimeConnectPayload`, etc.).

### Client and Server

```mermaid
sequenceDiagram
    participant C as IPC Client (CLI / TUI)
    participant S as IPC Server (Daemon)

    C->>S: connect Unix socket
    S-->>C: accept connection

    rect rgb(40, 60, 90)
        Note over C,S: One DaemonRequest / DaemonResponse exchange
    end

    C->>S: write JSON request + newline
    S->>S: parse DaemonRequest
    S->>S: dispatch_request
    S->>S: supervisor handler
    S->>S: build DaemonResponse
    S-->>C: write JSON response + newline
    C->>C: read JSON response + newline
```

### Transport Routing

`app/daemon/ipc/handler/dispatch.rs` maps every `DaemonRequestKind` variant to a
`*_response_via_supervisor` helper. Those helpers live in
`app/daemon/ipc/transport/` grouped by feature:

```mermaid
flowchart TD
    classDef req    fill:#1a2c3a,stroke:#5b8def,color:#e6edf3
    classDef route  fill:#2a1a3a,stroke:#b070df,color:#e6edf3
    classDef trans  fill:#2e2a1a,stroke:#dfba5b,color:#e6edf3

    REQ["DaemonRequestKind"]:::req
    TYPE{"dispatch"}:::route
    TS["transport/ping_shutdown.rs"]:::trans
    TP["transport/proxy.rs"]:::trans
    TR["transport/runtime.rs"]:::trans

    REQ --> TYPE
    TYPE -- "DaemonPing<br/>DaemonShutdown" --> TS
    TYPE -- "ProxyStart<br/>ProxyStop<br/>ProxyStatus" --> TP
    TYPE -- "RuntimeConnect<br/>RuntimeDisconnect<br/>RuntimeReplace<br/>RuntimeStatus" --> TR
```

## Health Checking

```mermaid
flowchart TD
    classDef tick    fill:#1a2744,stroke:#4a9eff,color:#e6edf3
    classDef check   fill:#2a1a3a,stroke:#b070df,color:#e6edf3
    classDef ok      fill:#1a3a1a,stroke:#5bdf8a,color:#e6edf3
    classDef warn    fill:#3a2a1a,stroke:#dfba5b,color:#e6edf3
    classDef fail    fill:#3a1a1a,stroke:#df5b5b,color:#e6edf3

    TICK["health tick fires"]:::tick
    CHECK{"active session?"}:::check
    PROBE["probe inbound SOCKS / HTTP ports"]:::check
    OPEN{"all inbounds reachable?"}:::check
    RECORD["record success<br/>reset failure count"]:::ok
    WAIT["skip"]:::ok
    INCR["increment failure count"]:::warn
    THRESH{"failures > threshold?"}:::warn
    TRIGGER["trigger rotation<br/>(HealthCheckFailed)"]:::fail
    COOLDOWN["enter cooldown period"]:::warn

    TICK --> CHECK
    CHECK -- "yes" --> PROBE
    CHECK -- "no"  --> WAIT
    PROBE --> OPEN
    OPEN  -- "yes" --> RECORD
    OPEN  -- "no"  --> INCR --> THRESH
    THRESH -- "yes" --> TRIGGER
    THRESH -- "no"  --> COOLDOWN
```

Inbound health is reported as `RuntimeInboundHealth` with per-endpoint
`RuntimeEndpointState::{Reachable, Unreachable, NotChecked}` (see
[Runtime Lifecycle](runtime-lifecycle.md)).

## Auto-Rotation

The daemon can rotate the active proxy on three triggers:

```rust
pub enum RotationTrigger {
    Manual,
    Timer,
    HealthCheckFailed,
}
```

The trigger is carried inline in `DaemonRequestKind::RuntimeReplace`. The
supervisor stores rotation state in `SupervisorState` (per-`AppContext`) and
reports it via `ProxyStatusPayload` (`rotation_enabled`, `interval_secs`,
`health_trigger_enabled`, `cooldown_secs`, `last_trigger`, `last_result`,
`cooldown_active`, `next_timer_epoch_secs`).

### Rotation Flow

```mermaid
flowchart TD
    classDef trigger fill:#1a2744,stroke:#4a9eff,color:#e6edf3
    classDef step    fill:#2a1a3a,stroke:#b070df,color:#e6edf3
    classDef ok      fill:#1a3a1a,stroke:#5bdf8a,color:#e6edf3
    classDef fail    fill:#3a1a1a,stroke:#df5b5b,color:#e6edf3
    classDef store   fill:#1a2e2e,stroke:#5bcfdf,color:#e6edf3

    TRIG{"trigger source"}:::trigger
    NEXT["select next candidate config"]:::step
    BUILD["handle_runtime_replace<br/>supervisor/handlers/runtime/"]:::step
    STOP_OLD["stop old process"]:::step
    SPAWN["spawn new Xray<br/>(configured ports)"]:::step
    WAIT_HEALTH["wait for inbound health"]:::step
    ATOMIC{"healthy?"}
    SWITCH["set active config"]:::ok
    CLEAN["record failed session<br/>clear active config"]:::fail
    PERSIST["persist new session record"]:::store

    TRIG -- "Timer"         --> NEXT
    TRIG -- "HealthCheckFailed" --> NEXT
    TRIG -- "Manual IPC"   --> NEXT
    NEXT --> BUILD --> STOP_OLD --> SPAWN --> WAIT_HEALTH --> ATOMIC
    ATOMIC -- "yes" --> SWITCH --> PERSIST
    ATOMIC -- "no"  --> CLEAN
```
