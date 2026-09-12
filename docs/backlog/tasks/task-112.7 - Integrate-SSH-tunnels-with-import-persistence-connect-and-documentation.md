---
id: TASK-112.7
title: Integrate SSH tunnels with import persistence connect and documentation
status: To Do
assignee: []
created_date: '2026-09-02 11:03'
labels:
  - ssh
  - import
  - database
  - cli
  - tui
  - docs
dependencies:
  - TASK-112.6
references:
  - TASK-112.5
  - TASK-112.6
  - src/model/protocol.rs
  - src/db
  - src/app/runtime_service
  - src/app/commands/output.rs
  - src/prober
documentation:
  - AGENTS.md
  - docs/src/06-architecture/import-pipeline.md
  - docs/src/03-features/importing.md
  - docs/src/03-features/deduplication.md
  - docs/src/03-features/runtime-management.md
  - docs/src/05-reference/database-schema.md
  - docs/src/02-cli/import.md
  - docs/src/02-cli/runtime.md
  - docs/src/02-cli/proxy.md
parent_task_id: TASK-112
priority: high
ordinal: 93000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Complete the user-facing SSH tunnel workflow on top of the managed backend. Add the approved typed model and persistence, safe configuration/import UX, deduplication, connect/test behavior, CLI/TUI/API presentation, status, deletion, and documentation while preserving the security contract from TASK-112.5.

Required reading before implementation:
- AGENTS.md
- TASK-112.5 and TASK-112.6
- docs/src/06-architecture/import-pipeline.md
- docs/src/03-features/importing.md
- docs/src/03-features/deduplication.md
- docs/src/03-features/runtime-management.md
- docs/src/05-reference/database-schema.md
- docs/src/02-cli/import.md
- docs/src/02-cli/runtime.md
- docs/src/02-cli/proxy.md
- Existing normalized Node, repositories, output helpers, daemon IPC, and prober abstractions

Do not advertise an ssh URI unless TASK-112.5 defines a safe, documented Xrat-specific configuration syntax. Prefer explicit flags or configuration files for key paths and trust material.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Users can create, inspect, connect, test, stop, and delete an SSH tunnel configuration through documented non-interactive CLI workflows
- [ ] #2 Persistence round trips the approved non-secret SSH fields and secret references without copying private-key contents or plaintext passwords into the database
- [ ] #3 Deduplication distinguishes endpoints and identities correctly while redacting sensitive paths or material from default output
- [ ] #4 Connect exposes a local HTTP and/or SOCKS entry point through the existing runtime conventions and test traffic traverses the SSH dynamic forward
- [ ] #5 CLI, TUI, HTTP API, daemon IPC, events, and JSON output represent SSH sessions consistently or explicitly document intentionally unsupported surfaces
- [ ] #6 Migration coverage works for SQLite and Postgres when schema changes are required, without editing released migrations
- [ ] #7 Import, runtime, proxy, configuration, protocol-matrix, and security documentation include setup, host-key enrollment, agent/key examples, failure recovery, and limitations
- [ ] #8 End-to-end regression tests and just fmt ci pass
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
