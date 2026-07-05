---
id: TASK-36
title: Extract Shared Port Waiter Abstraction
status: To Do
assignee: []
created_date: '2026-07-05 14:43'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-4
dependencies: []
priority: medium
ordinal: 16
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/3-ports/16-port-waiter-abstraction.md`

# Extract Shared Port Waiter Abstraction

## Finding

### [Priority: Medium] Extract shared TCP port-waiting abstraction

**Files involved:**

- `src/xray/process_mgmt/process.rs:74-99`
- `src/xray/process/spawn.rs:63-83`
- `src/singbox/process_mgmt.rs:89-113`

**Problem:** The same `TcpStream::connect` + `Instant::now` polling loop is
duplicated in three engine startup implementations. Each polls a set of TCP
ports until they accept a connection or a timeout elapses. The logic is
identical but independently implemented: same retry interval, same overall
timeout calculation, same error reporting.

**Why this change is needed:** Three copies of the same polling loop means bug
fixes or tuning (retry interval, timeout behavior, error messages) must be
applied in three places. It also prevents testing readiness logic without
binding real TCP ports. The duplication is a clear violation of DRY in a
critical startup path.

**How to implement it:** Extract a `PortWaiter` trait and a default production
implementation. Provide methods for waiting on a single port or multiple ports.
Replace the three inline polling loops with calls to this shared abstraction.
Add a `MockPortWaiter` that returns simulated latency or timeout for testing.

**Positive effect on the codebase:** Engine startup tests can verify timeout
behavior, success timing, and partial-failure handling without real ports. The
three engine modules become smaller and easier to reason about.

**Suggested target architecture:** `PortWaiter` as a utility port in
`src/support/` or `src/app/ports/`. Used by `ProcessSpawner` consumers during
the readiness-check phase.

**Risk / migration notes:** Low risk. Pure extraction with no behavior change.
Add tests for the shared implementation first, then replace each inline loop one
at a time, verifying startup still works after each replacement.
<!-- SECTION:DESCRIPTION:END -->
