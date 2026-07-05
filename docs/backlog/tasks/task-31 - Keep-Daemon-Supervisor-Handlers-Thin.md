---
id: TASK-31
title: Keep Daemon Supervisor Handlers Thin
status: To Do
assignee: []
created_date: '2026-07-05 14:43'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-3
dependencies: []
priority: medium
ordinal: 5
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/2-use-cases/5-thin-daemon-supervisor-handlers.md`

# Keep Daemon Supervisor Handlers Thin

## Finding

### [Priority: High] Keep daemon supervisor handlers thin

**Files involved:**

- `src/app/daemon/supervisor/handlers/runtime/runtime_status_connect.rs`
- `src/app/daemon/supervisor/handlers/runtime/runtime_lifecycle/replace.rs`
- `src/app/daemon/supervisor/handlers/runtime/runtime_lifecycle/disconnect.rs`
- `src/app/daemon/supervisor/types.rs`
- `src/app/runtime_service/replace_flow`

**Problem:** Daemon supervisor handlers do more than dispatch. They update
runtime transition metadata, maintain rotation state fields, record events,
construct IPC payloads, and translate runtime failures. This logic is
interleaved with supervisor message handling.

**Why this change is needed:** Rotation and runtime transition rules are
application behavior. Keeping them in daemon handlers makes them hard to reuse
from CLI/TUI/API flows and hard to unit-test without supervisor channels. It
also makes debugging harder because state changes, event recording, and IPC
response construction happen in one async path.

**How to implement it:** Extract runtime transition and rotation orchestration
into application services, for example `RuntimeTransitionService` and
`RotationService`. These services should accept typed requests, update metadata,
call `RuntimeService`, record events, and return typed outcomes. Keep daemon
handlers responsible for channel receive/send, daemon-specific state fields, and
mapping outcomes to IPC payloads.

**Positive effect on the codebase:** Daemon behavior becomes easier to reason
about, rotation can be tested without IPC plumbing, and future interfaces can
reuse the same transition metadata and event behavior.

**Suggested target architecture:** Supervisor code manages scheduling and
channels; application services own runtime/rotation use-cases; event persistence
is best-effort inside the use-case layer with structured results.

**Risk / migration notes:** Medium risk because rotation state is subtle. Add
regression tests around manual replace, timer replace, health cooldown, and
metadata updates before moving logic.
<!-- SECTION:DESCRIPTION:END -->
