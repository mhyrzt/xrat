---
id: TASK-112.5
title: Define the managed SSH tunnel contract and security model
status: To Do
assignee: []
created_date: '2026-09-02 11:02'
labels:
  - ssh
  - security
  - protocols
  - architecture
dependencies:
  - TASK-107
references:
  - 'https://man.openbsd.org/ssh'
  - 'https://man.openbsd.org/OpenBSD-current/man5/ssh_config.5'
  - 'https://github.com/XTLS/Xray-core/issues/5682'
  - TASK-107
documentation:
  - AGENTS.md
  - docs/src/06-architecture/import-pipeline.md
  - docs/src/06-architecture/runtime-lifecycle.md
  - docs/src/06-architecture/daemon-architecture.md
  - docs/src/05-reference/config-file.md
parent_task_id: TASK-112
priority: high
ordinal: 91000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Turn the broad SSH scope question into an implementation-ready contract for first-class dynamic SOCKS forwarding. Decide the backend boundary, supported input/configuration shape, authentication sources, host-key trust workflow, secret handling, platform expectations, lifecycle semantics, and explicit non-goals before process code is written.

Required reading before implementation:
- AGENTS.md
- docs/src/06-architecture/import-pipeline.md
- docs/src/06-architecture/runtime-lifecycle.md
- docs/src/06-architecture/daemon-architecture.md
- docs/src/05-reference/config-file.md
- OpenSSH ssh(1) and ssh_config(5) manuals
- Xray SSH outbound feature request showing SSH is not an Xray protocol
- TASK-107

The initial capability should target SSH dynamic forwarding as a general SOCKS upstream. Local and remote fixed-port forwarding, VPN mode, shell sessions, and an invented public ssh share-link format remain out of scope unless this task explicitly justifies and splits them.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 An architecture decision selects managed OpenSSH subprocess or an in-process SSH library using security, portability, maintenance, and feature criteria
- [ ] #2 The input contract defines host, port, user, identity or agent source, host-key policy, known-hosts location, bind address, local SOCKS port policy, keepalive, connect timeout, and optional jump-host behavior
- [ ] #3 The contract forbids plaintext password persistence, password-in-URI support, shell command construction, disabled host-key verification defaults, and secret-bearing logs
- [ ] #4 First-contact and changed-host-key behavior is non-interactive, explicit, testable, and resistant to man-in-the-middle downgrade
- [ ] #5 Dynamic SOCKS forwarding is distinguished from fixed local or remote forwarding and from native Xray outbound support
- [ ] #6 Platform support and behavior when OpenSSH or required agent/key material is unavailable are documented
- [ ] #7 Review-sized implementation follow-ups and migration implications are confirmed before this task is finalized
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
