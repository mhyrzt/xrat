# tun

Apply managed TUN capture to the current connection and inspect its readiness.

```bash
xrat tun <subcommand>
```

TUN capture creates a network interface and system routes, so the engine binary
needs `CAP_NET_ADMIN`. On Linux this is a file capability, granted with
`setcap`. See [TUN Capture](../03-features/runtime-management.md#tun-capture)
for the full workflow and caveats.

## enable / disable

Enable TUN on the current connected config, or return it to local proxy mode:

```bash
xrat tun enable
xrat tun enable --kill-switch
xrat tun disable
xrat tun enable --json
```

The runtime owner checks engine support, capabilities, interface ownership, and
native config validity before stopping the current session. It reconnects the
same config in the requested mode and saves `runtime.tun.enabled` only after a
successful change. Startup or save failures attempt to restore the previous mode
and connection. Repeating a command does not restart an already matching
session. When disconnected, the setting is saved for the next connect.

Normal toggles do not require setup or daemon restarts. First-time privileges,
replacement binaries, or upgrading a daemon from a version without live toggles
may require setup and one restart; the error explains the next step. A CLI
command never takes over a standalone TUI's running session: press `U` in that
TUI instead. A global `--config` override must match the daemon's config file.

In the TUI, press `U` for the same checked live toggle. Its result appears in
the status message and log. You can also change only `runtime.tun.enabled` in
settings and save with `Ctrl+S` to apply immediately. Save other settings
separately when changing the enabled flag; their existing restart workflow still
applies.

## status

Report TUN readiness for the current configuration:

```bash
xrat tun status
```

```text
TUN status
engine                xray 26.7.11
active capture        yes
active config         a1b2c3
active interface      xrat0
engine check          ready
configured TUN        yes
configured interface  xrat0
systemd service       ready (NoNewPrivileges=false override present)
engine file           /home/user/.local/share/xrat/cores/xray/xray  ready (cap_net_admin,cap_net_raw=ep)
xrat file             /usr/local/bin/xrat  ready (cap_net_admin,cap_net_raw=ep)
```

`--json` prints the same data as structured JSON. `tun_enabled` is the saved
setting; `tun_active` requires a running session and an owned kernel TUN device
with a matching interface index. It does not prove external traffic or DNS
reachability. `ready` covers engine support and privilege checks; it is separate
from active capture.

## setup

Grant `CAP_NET_ADMIN` and `CAP_NET_RAW` to the files TUN needs and configure
systemd service permissions:

```bash
xrat tun setup
```

On Linux the selected engine receives
`cap_net_admin,cap_net_raw,cap_dac_read_search,cap_sys_ptrace+ep` for capture
and process matching. XRAT receives only `cap_net_admin,cap_net_raw+ep` for
cleanup. On Linux systems with an installed systemd user service, it also
automatically installs a drop-in override
`~/.config/systemd/user/xrat-daemon.service.d/10-tun.conf`
(`NoNewPrivileges=false`) and reloads the user daemon
(`systemctl --user daemon-reload`). Use `--dry-run` to print the commands and
planned file changes without running them.

Restart an installed systemd daemon with
`systemctl --user restart xrat-daemon.service` so it picks up the override and
capabilities. For a standalone daemon, use `xrat daemon restart`.

Capability restoration uses normal sudo authentication and may prompt for a
password. XRAT does not install passwordless sudoers rules. If you previously
used the removed `--sudoers` helper, review and remove `/etc/sudoers.d/xrat-tun`
with `sudo rm /etc/sudoers.d/xrat-tun` to revoke its persistent grant.

File capabilities are automatically restored during `xrat upgrade` (and core
reinstalls via `xrat install`) when TUN was previously configured (see
`auto_setup_on_upgrade` in settings), so manual re-setup is only required if
auto-restoration is disabled or failed.

## Requirements

- Linux: Xray >= 26.7.11 or sing-box, with `libcap` tools (`setcap`/`getcap`).
- macOS: sing-box, a root runtime owner and a numbered `utun` interface.
- Windows: protected LocalSystem machine service; Xray >= 26.9.30 and < 27.0.0,
  or sing-box >= 1.13.21 and < 1.14.0.
- V2Ray TUN is unsupported.

## Windows preparation

In an elevated administrator shell, install the machine service, stop it, then
prepare the protected driver files:

```powershell
xrat --profile machine daemon stop
xrat --profile machine tun setup --dry-run
xrat --profile machine tun setup
```

Setup downloads the official Wintun 0.14.1 archive over HTTPS. It verifies
pinned SHA-256 hashes for the archive, the native x64 or ARM64 DLL, and the
license. Windows Authenticode verification checks the staged DLL against the
local trust policy without network retrieval or a fresh revocation check. Files
are staged with administrator ownership in the protected machine core directory
and activated beside `xray.exe`; the original license is retained as
`wintun-LICENSE.txt`. Repeating setup with verified files performs no download.
The service must be stopped and remains stopped. Setup does not create a network
adapter, change routes or DNS, or enable capture.

Windows capture runs only in the protected machine service. Configure its
protected engine and TUN addresses as administrator, then start the service and
use the ordinary registered operator account:

```powershell
xrat --profile machine daemon start
xrat --profile machine tun enable --json
xrat --profile machine tun status --json
xrat --profile machine tun disable
```

For sing-box automatic routing, set `runtime.tun.strict_route = true` before
enabling capture. Split policy changes require administrator maintenance;
ordinary machine CLI toggles cannot change that policy. User-profile capture is
rejected. Routine toggles do not download drivers or change driver policy.

Startup verifies the Wintun device identity, configured interface addresses,
per-interface DNS, and capture-route coverage before reporting active capture.
Conflicting foreign VPN routes or overlapping addresses preserve the current
connection. Cleanup waits for the engine-owned adapter and routes to disappear;
it retains the ownership journal on failure and never deletes unrelated adapters
or rewrites physical-interface DNS. Service crash containment terminates child
engines; service startup reconciles stale ownership and remains disconnected
until explicitly requested.

These Windows paths are implemented, but native x64/ARM64 traffic, DNS,
replacement, crash and reboot acceptance remains pending. Driver preparation or
local compilation alone does not establish full Windows support.

When a daemon is running, `tun status` also inspects its effective
`CAP_NET_ADMIN` and `NoNewPrivs` through the daemon socket peer PID. The JSON
`daemon` object contains its PID, readiness and status; it is `null` when no
daemon is reachable. A drop-in on disk alone does not make an already-running
daemon ready.

## Split tunneling

`whitelist` sends selected applications through the proxy and others direct.
`blacklist` sends selected applications direct and others through the proxy.
`all` captures all applications. DNS interception and private/LAN routing retain
their precedence. Names, executable paths and application directories match as
alternatives.

Desktop selections preserve their desktop ID, real executable and companion
workers. Executable symlinks include their canonical targets. Shell wrappers are
inspected for literal `exec /path/to/app` statements and are never executed.
Variable-based wrappers require an explicit executable or application directory.
Inspect resolved rules and warnings with `xrat tun split list` before relying on
application matching. `xrat tun split status` also shows wrapper warnings and
the compiled engine matchers.

## Kill Switch

The TUN kill switch provides fail-closed isolation for mode `all`: no packets
leave the host via physical network interfaces using the real IP address if the
proxy engine crashes, stops unexpectedly, or drops the connection.

> **Important**: The kill switch requires `split_mode = "all"`. Split tunneling
> (`whitelist` or `blacklist`) deliberately allows traffic to bypass the tunnel
> directly and is rejected when the kill switch is enabled.

```bash
# Check current kill switch status
xrat tun killswitch status

# Enable kill switch (auto mode)
xrat tun killswitch on

# Disable kill switch
xrat tun killswitch off

# Emergency reset (flushes firewall tables and resets state)
xrat tun killswitch reset
```

The firewall backend defaults to Linux `nftables` (`table inet xrat_killswitch`)
with automatic fallback to `iptables` / `ip6tables` (`XRAT_KILLSWITCH` chain).
Rules isolate egress so that only traffic over loopback (`lo`), the TUN interface
(`xrat0`), packets marked with the engine socket mark (`0x162`), proxy endpoint
destinations, and DHCP/ND/LAN subnets (when `allow_lan = true`) are accepted; all
other egress packets are dropped.

Modes:
- `off`: No kill switch firewall rules are loaded.
- `auto`: Firewall rules are engaged when TUN session starts and disengaged on
  explicit `disconnect` or `tun disable`. On engine crash or unexpected termination,
  the fail-closed rules remain active.
- `strict`: Firewall rules remain engaged even after manual disconnect until
  explicitly turned off or reset.

## macOS

Set `runtime.engine = "sing-box"`. Managed capture defaults to `utun99` and
requires a root runtime owner with explicit config and database paths:

```bash
sudo xrat --config /absolute/config.toml --database /absolute/db.sqlite daemon start
```

`xrat tun setup` prints instructions; it does not install a root launchd
service. The installed user launchd service cannot acquire root privileges.
Explicit paths ensure the privileged process uses your configuration.

Application discovery searches `/Applications`, `/System/Applications` and
`~/Applications`. A bundle directory such as `/Applications/Firefox.app/`
includes nested helper processes. Running apps use executable paths or their
enclosing bundle directories.

`strict_route` and `route_exclude_address` are rejected on macOS. Startup
refuses VPN default routes or conflicting capture routes. Cleanup waits for the
engine to release its `utun` socket, retaining ownership if release fails. macOS
traffic, DNS and restoration acceptance requires a native privileged run; Linux
checks do not establish that runtime proof.
