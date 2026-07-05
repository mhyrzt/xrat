---
id: DRAFT-1
title: 'Hard, P3: Runtime TUN mode (system-wide transparent proxy)'
status: Draft
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
Legacy path: `docs/backlog/feature/runtime-tun-mode.md`

# Hard, P3: Runtime TUN mode (system-wide transparent proxy)

### Status

Draft

### Goal

Let xrat capture and proxy a host's traffic at the network layer through a TUN
virtual interface, so traffic from arbitrary apps is routed through the active
config without per-app SOCKS/HTTP configuration.

### Background / why this is split out

This was raised alongside the runtime interface-binding work (see
[runtime-network-interface.md](runtime-network-interface.md)) but is far larger
in scope than outbound/inbound interface selection, so it ships separately.

TUN mode is not a config-generation tweak. It requires:

- A TUN device and OS routing changes (default-route hijack or policy routing),
  which need elevated privileges (root / `CAP_NET_ADMIN` on Linux).
- An engine that supports a `tun` inbound. **Xray has no native `tun` inbound.**
  Options:
  - Use the **sing-box `tun` inbound**, which xrat already builds for hy2 but
    would need to support for general protocols and as an explicit user mode.
  - Or run an external **tun2socks** process in front of xrat's SOCKS inbound.
- DNS handling (hijack/redirect) so name resolution does not leak.
- Teardown that reliably restores routing and DNS when the session stops or
  crashes, including on unexpected exit.
- Platform-specific behavior across Linux/macOS/BSD/Windows.

### Open questions

- sing-box `tun` inbound vs external tun2socks: which becomes the supported path?
- How are privileges acquired (run as root, setcap, a privileged helper)?
- Default-route hijack vs policy routing, and how to scope which traffic enters
  the tunnel (allow/exclude lists, app/uid rules).
- DNS strategy (fake-ip, hijack, or system resolver passthrough).
- How does TUN interact with the daemon/systemd user-service model, which is
  currently unprivileged?
- Stop/restore guarantees and crash recovery.

### Possible config shape

```toml
[runtime.tun]
enabled = false
interface = "xrat-tun"
# stack, mtu, auto_route, dns hijack, exclude routes, etc. — TBD per engine
```

### Changes required (high level)

- Decide engine path (sing-box tun vs tun2socks) and design the runtime launch
  flow for a privileged inbound.
- Privilege model and a safe routing/DNS setup + guaranteed teardown.
- New `[runtime.tun]` config types/defaults/validation.
- Runtime service support for a TUN session distinct from SOCKS/HTTP inbounds.
- Extensive docs covering privileges, platform caveats, and recovery.

### Verification

- Config parsing/default tests.
- Manual end-to-end on Linux: enable TUN, confirm unconfigured apps egress
  through the proxy, confirm DNS does not leak, confirm routing/DNS restore on
  stop and on crash.
<!-- SECTION:DESCRIPTION:END -->
