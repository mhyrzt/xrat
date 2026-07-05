---
id: TASK-43
title: Add Ports For External Dependencies
status: To Do
assignee: []
created_date: '2026-07-05 14:44'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-4
dependencies: []
priority: medium
ordinal: 7
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/3-ports/7-external-dependency-ports.md`

# Add Ports For External Dependencies

## Finding

### [Priority: Medium] Add ports for external dependencies

**Files involved:**

- `src/app/context.rs`
- `src/app/app_paths.rs`
- `src/app/input/source.rs`
- `src/tui/data/mod.rs`
- `src/app/commands/scan/mod.rs`
- `src/app/runtime_service`
- `src/xray/process_mgmt`

**Problem:** Concrete database, filesystem, environment, network, process, and
IPC calls are spread across application and adapter modules. `AppContext::build`
connects a real database, `app_paths` reads environment variables and writes
config files, TUI data probing runs runtime binaries, scan calls `tcp_check`
directly, and runtime services use concrete process-management modules.

**Why this change is needed:** Hidden concrete I/O makes use-cases hard to
unit-test and encourages tests to create temp directories, real databases,
sockets, or subprocesses. It also makes failure simulation difficult, which
hurts debugging and reliability work.

**How to implement it:** Introduce focused traits for key boundaries:
`ConfigRepository`, `RuntimeProcessManager`, `InputReader`, `NetworkProbe`,
`DaemonClient`, `Clock`, and `Filesystem`. Keep trait sets small and use
concrete adapters for production. Add an `AppServices` or `App` factory that
wires repositories and ports from `AppContext`. Start with runtime control and
config/test use-cases instead of wrapping every database method at once.

**Positive effect on the codebase:** Core services become testable with fakes,
adapter code becomes clearer, and failure cases can be tested without slow or
flaky integration setup.

**Suggested target architecture:** Application services depend on ports;
infrastructure modules implement ports; the main binary, daemon runner, router
factory, and TUI runner compose concrete dependencies.

**Risk / migration notes:** Do this incrementally to avoid trait sprawl. Add
ports only where a use-case needs test seams or adapter reuse.
<!-- SECTION:DESCRIPTION:END -->
