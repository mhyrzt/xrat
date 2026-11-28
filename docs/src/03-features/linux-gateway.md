# Linux IPv4 gateway

This guide describes the development implementation of XRAT gateway mode.
Managed Xray and sing-box passed TCP, UDP and DNS tests through an isolated
x86-64 Linux guest. Raspberry Pi deployment still needs native ARM64 packet
and reboot acceptance with the selected engine. See the
[acceptance matrix](../06-architecture/gateway.md#verification-and-traceability-matrix).

## Raspberry Pi setup

Use a 64-bit Debian-based Raspberry Pi OS installation on the Pi Zero 2 W.
Keep the existing router in charge of Wi-Fi and DHCP. Give the Pi a stable IPv4
address on the client network. The Pi and upstream router can share one
interface, such as `wlan0` on a Pi Zero 2 W. The router must allow Wi-Fi
stations to reach one another. Keep console access while preparing the
network and validate the path with one client first.

Install XRAT and the selected native Xray or sing-box core. Confirm that Linux
provides `/dev/net/tun`, `ip` from iproute2 and `nft` from nftables. Run
installation and the managed runtime owner with access to the same XRAT
configuration and runtime directory. Both need permission to inspect and
manage the Linux network. Engine file capabilities alone do not grant these
permissions to the runtime owner.

Keep IPv4 forwarding disabled globally, for new interfaces, and on the LAN at
boot. XRAT enables forwarding on the LAN and capture interfaces only after
installing the guard. Disable IPv4 redirects both globally and on the LAN.
Check per-interface forwarding and redirect settings after boot, including any
settings applied by a network manager. Keep nftables flowtable offload disabled.
An external firewall manager that flushes the XRAT table can interrupt
protection. Check the diagnostic output after any such changes.

For a Pi at `192.168.1.2` on a `192.168.1.0/24` LAN, use settings like these:

```toml
[gateway]
interface = "wlan0"
lan_subnet = "192.168.1.0/24"

[runtime]
engine = "sing-box"

[runtime.tun]
enabled = true
auto_route = false
```

Use `engine = "xray"` with a generated Xray node. Keep an enabled local
readiness listener, such as SOCKS, with its existing loopback bind. Gateway
mode needs all-traffic TUN mode, the host kill switch disabled, and FakeIP
disabled. Complete raw Xray profiles are rejected in gateway mode because XRAT
cannot guarantee their routing policy. Test the engine version on the
deployment host. The architecture matrix lists the acceptance pins.

```bash
xrat gateway doctor
xrat gateway install --dry-run
xrat gateway install
xrat gateway status --json
xrat connect <config-id>
xrat gateway status
```

Installation creates the closed forwarding guard. A successful connection adds
capture routing and opens the path to the owned TUN. Configure the test client's
**IPv4** default gateway to point to the Pi. Leave other clients on their usual
gateway until the test client passes the checks below.

Clients reach other devices within their own subnet directly. Additional
`local_subnets` exceptions require an exact IPv4 subnet directly attached to the
selected gateway LAN interface. XRAT scopes the exception to that interface
and subnet. A broad private address range does not qualify merely because its
addresses are private.

## DNS and IPv6

UDP and TCP traffic to an off-LAN IPv4 DNS resolver follows the proxy route.
Choose an IPv4 resolver reachable through the selected proxy. Verify both
transports from the client. The XRAT DNS listener serves the gateway host over loopback. The runtime resolves the proxy endpoint using the host's
bootstrap path.

A resolver on the client's own LAN can receive queries directly. A configured
local-subnet exception can also send queries to that subnet without the proxy.
A client can use another network interface, DNS path, or independent IPv6
default route. Gateway mode controls IPv4 forwarding that reaches the Pi. It
does not configure client IPv6 or enforce DNS for traffic that never arrives.

## Verify, disconnect, and remove

Check that the client can reach the Pi and router. Test TCP and UDP to a known
off-LAN endpoint and check the observed source at that endpoint. Run DNS
lookups over UDP and TCP. While testing, capture traffic at the physical
upstream and distinguish the proxy connection from any direct client traffic.
After `xrat disconnect`, off-LAN client traffic should stop while access to the
Pi remains available. Repeat during a proxy crash, replacement and reboot
before relying on the setup.

On a development Linux host with network namespaces and an accessible
`/dev/net/tun`, run the managed-engine acceptance test from the source checkout:

```bash
just runtime-gateway /path/to/xrat /path/to/xray /path/to/sing-box /path/to/receipts
```

The test records controlled SOCKS and origin observations for TCP, UDP, and DNS.
Run the Raspberry Pi reboot checks separately on native ARM64 hardware.

A status such as `closed_guard_observed` or `active_guard_observed` reports
kernel rule read-back. Use client packet tests to verify forwarding separately. If the status is degraded or unknown,
inspect `xrat gateway doctor` and the daemon logs.

Move clients back to the router before removing XRAT gateway mode. Disconnect
the managed session and preview removal before applying it:

```bash
xrat disconnect
xrat gateway remove --dry-run
xrat gateway remove
```

Removal requires the session to be inactive and forwarding disabled. If a rule
or route has changed externally, keep the ownership files and use the
diagnostics to identify the conflict. Avoid flushing unrelated firewall tables
or deleting ownership files during recovery.
