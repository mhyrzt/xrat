---
id: TASK-112.6
title: Implement the managed SSH dynamic-forward runtime backend
status: To Do
assignee: []
created_date: '2026-09-02 11:03'
labels:
  - ssh
  - runtime
  - process-management
  - security
dependencies:
  - TASK-112.5
references:
  - TASK-112.5
  - 'https://man.openbsd.org/ssh'
  - 'https://man.openbsd.org/OpenBSD-current/man5/ssh_config.5'
  - src/xray/process_mgmt
  - src/app/events.rs
documentation:
  - AGENTS.md
  - docs/src/06-architecture/runtime-lifecycle.md
  - docs/src/06-architecture/daemon-architecture.md
parent_task_id: TASK-112
priority: high
ordinal: 92000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement the approved SSH backend as a managed runtime component that establishes a loopback-bound dynamic SOCKS proxy and participates in Xrat lifecycle, status, cancellation, and event recording. Follow the exact contract from TASK-112.5 rather than expanding SSH modes during implementation.

Required reading before implementation:
- AGENTS.md
- TASK-112.5 and its architecture decision
- docs/src/06-architecture/runtime-lifecycle.md
- docs/src/06-architecture/daemon-architecture.md
- OpenSSH ssh(1) and ssh_config(5) manuals
- Existing Xray and sing-box process-management modules
- src/app/events.rs requirements for best-effort diagnostic events
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The backend launches SSH without a shell, binds dynamic forwarding to loopback by default, requests no remote command, and fails when forwarding setup fails
- [ ] #2 Readiness is based on successful forward establishment and a bounded SOCKS probe rather than process existence alone
- [ ] #3 Stop, cancellation, daemon shutdown, failed authentication, host-key failure, port conflict, and unexpected child exit leave no orphan process or stale runtime session
- [ ] #4 Key, agent, known-hosts, keepalive, timeout, and approved jump-host settings follow TASK-112.5 exactly and secrets are redacted from arguments shown to users, logs, errors, and events
- [ ] #5 Runtime status and best-effort lifecycle events identify SSH sessions and actionable failure causes without exposing sensitive material
- [ ] #6 Unit tests cover command argument construction and redaction; integration tests use an isolated SSH fixture or explicitly documented test harness
- [ ] #7 Supported platforms pass focused lifecycle tests together with just fmt ci
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
