---
id: TASK-14
title: 'Medium, P3: Runtime network interface binding setting'
status: Done
assignee: []
created_date: '2026-07-05 14:43'
labels:
  - legacy-import
  - feature
dependencies: []
priority: low
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/feature/runtime-network-interface.md`

# Medium, P3: Runtime network interface binding setting

### Status

Implemented (outbound interface bind, outbound source bind_address, inbound
listen interface). TUN mode was split into
[runtime-tun-mode.md](runtime-tun-mode.md).

### Goal

Add an explicit config option for choosing the network interface or outbound
bind address used by xrat-managed runtime traffic.

### Implementation notes

Shipped as `[runtime.network]` with `interface` (Xray `sockopt.interface`,
`SO_BINDTODEVICE`), `mark` (Xray `sockopt.mark` fwmark), `bind_address`, and
`listen_interface` (resolved to an address for managed inbound `listen`).

Known limitation: the Xray engine has no source-IP `sockopt`, so `bind_address`
is validated and stored but ignored by Xray at runtime (a warning is logged).
A future sing-box mapping could honor it.

### Motivation

Some users may need xrat traffic to leave through a specific interface, for
example Wi-Fi vs Ethernet, VPN vs physical interface, or a policy-routed link.
Today runtime generation does not expose a user-facing interface selection knob.

### Open questions

- Should the user configure an interface name, a source IP address, or both?
- Should this apply to all runtime engines or only Xray first?
- Should probe/test traffic inherit the runtime bind setting or remain host
  default to avoid skewing measurements?
- What is the expected behavior when the configured interface disappears?

### Possible config shape

```toml
[runtime.network]
interface = ""
bind_address = ""
```

Prefer disabled/empty defaults so existing runtime configs remain unchanged.

### Changes required

- Research how Xray and sing-box express outbound interface/source binding.
- Add runtime config types/defaults under `src/app/config/` once the engine
  mapping is clear.
- Validate that configured interface/address values are syntactically sane.
- Thread settings into runtime config generation without persisting generated
  root-level engine JSON.
- Document platform caveats for Linux/macOS/BSD behavior.

### Verification

- Config parsing/default tests.
- Runtime config generation tests proving default output is unchanged.
- Engine-specific generation tests when interface or bind address is set.
- Manual verification on a host with at least two network paths.
<!-- SECTION:DESCRIPTION:END -->
