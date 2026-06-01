# Daemon Architecture

The daemon is a background process that owns the managed Xray runtime, accepts
IPC requests from CLI/TUI clients, runs health checks, and drives auto-rotation.

## Process Model

```mermaid
graph TB
    CLI[xrat daemon start]
    PARENT["Parent: validates config, forks child"]
    CHILD["Child: execs 'xrat daemon run-server'"]
    SOCK[Unix socket /path/to/xrat.sock]
    CLIENT1[xrat runtime status]
    CLIENT2[xrat connect]
    CLIENT3[xrat proxy start]

    CLI --> PARENT
    PARENT -- "std::process::Command" --> CHILD
    CHILD -- creates --> SOCK
    CLIENT1 -- IPC request --> SOCK
    CLIENT2 -- IPC request --> SOCK
    CLIENT3 -- IPC request --> SOCK
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
    REQ[DaemonRequestKind]
    REQ --> TYPE{Which request?}
    TYPE -- DaemonPing --> TS[transport/ping_shutdown.rs]
    TYPE -- DaemonShutdown --> TS
    TYPE -- ProxyStart --> TP[transport/proxy.rs]
    TYPE -- ProxyStop --> TP
    TYPE -- ProxyStatus --> TP
    TYPE -- RuntimeConnect --> TR[transport/runtime.rs]
    TYPE -- RuntimeDisconnect --> TR
    TYPE -- RuntimeReplace --> TR
    TYPE -- RuntimeStatus --> TR
```

## Health Checking

```mermaid
flowchart TD
    TICK[Health tick fires]
    CHECK{Active session?}
    PROBE[Probe inbound SOCKS / HTTP ports]
    OPEN{All inbounds reachable?}
    RECORD[Record success]
    INCR[Increment failure count]
    THRESH{Failures > threshold?}
    TRIGGER[Trigger rotation HealthCheckFailed]
    COOLDOWN[Enter cooldown period]
    WAIT[skip / wait]

    TICK --> CHECK
    CHECK -- yes --> PROBE
    CHECK -- no --> WAIT
    PROBE --> OPEN
    OPEN -- yes --> RECORD
    OPEN -- no --> INCR
    INCR --> THRESH
    THRESH -- yes --> TRIGGER
    THRESH -- no --> COOLDOWN
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
    TRIG{Trigger source}
    TRIG -- Timer fired --> NEXT[Select next candidate config]
    TRIG -- Health failed --> NEXT
    TRIG -- Manual IPC --> NEXT
    NEXT --> BUILD["handle_runtime_replace (supervisor/handlers/runtime/runtime_lifecycle)"]
    BUILD --> SPAWN[Spawn new Xray with new ports]
    SPAWN --> WAIT_HEALTH[Wait for inbound health]
    WAIT_HEALTH --> ATOMIC{Healthy?}
    ATOMIC -- yes --> SWITCH[Atomically swap active config]
    ATOMIC -- no --> CLEAN[Kill new process, keep old]
    SWITCH --> STOP_OLD[Stop old process]
    STOP_OLD --> PERSIST[Persist new session record]
```
