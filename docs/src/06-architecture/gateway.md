# Linux IPv4 gateway architecture

**Status: architectural contract; gateway implementation and packet validation
pending.** This is the design contract for
[issue #294](https://github.com/mhyrzt/xrat/issues/294), not an installation
guide or a claim that gateway mode is supported. Engine-specific mechanisms and
the open decisions below require native and packet-level tests before this
architecture can be considered validated.

The initial `xrat gateway` CLI provides configuration parsing and partial
read-only preflight. Gateway installation, removal and protection are not yet
available. Its status is unknown; it does not install a guard or activate the
runtime. `doctor`, `status`, and attempted dry-runs do not initialize the XRAT
database or modify networking. The installation and removal commands fail until
the required ownership and protection implementations are in place.

The proposed settings can be parsed from the existing configuration file:

```toml
[gateway]
interface = "eth0"
lan_subnet = "192.168.1.0/24"
local_subnets = ["192.168.2.0/24"]
```

Install flags override the file; repeated `--local-subnet` values replace its
list for that operation. Discovery reports candidate IPv4 interfaces, main-table
default routes, and IPv4 policy rules, but cannot yet verify all routing tables
or the complete topology. It checks the configured engine candidate against the
existing host TUN version gate; the selected proxy backend and gateway capability
remain unverified. Caller privileges are separate from privileges observed for
the daemon socket peer; the peer's identity and the engine's inherited
capabilities are not established by that observation.
The partial conflict report observes nftables base-chain hooks, policy rules
with marks, selected host TUN ownership and IPv4 sysctls. A successful read is
not a conflict-free verdict: gateway marks, firewall ownership, iptables
interaction and boot recovery are not yet established. Failed or denied
inspection is reported as unknown.
The ignored `gateway_preflight_read_only` namespace test exercises observation
of a foreign nftables base chain, a marked policy rule and a conflicting
interface, then compares networking and IPv4 sysctl snapshots before and after
doctor and a rejected install dry-run. It does not validate forwarding,
gateway protection, reboot recovery or interaction with a running proxy.
Checks that local exceptions cannot permit direct internet forwarding remain
unimplemented. CIDR syntax validation alone is not a safety check.

## Purpose and scope

An XRAT network gateway lets selected LAN clients send IPv4 internet traffic
through an XRAT-managed proxy without configuring each application to use a
proxy listener. It is distinct from the machine-service gateway, which provides
access to XRAT's service and is not a router for LAN packets.

The MVP targets Linux, including a Raspberry Pi Zero 2 W with a Debian-based
system. Both Xray and sing-box are independent target backends; neither has
gateway support until its own acceptance checks pass. The existing router
provides LAN connectivity, DHCP and upstream internet access. XRAT does not
configure the router, DHCP, Wi-Fi access points or a full IPv6 gateway.

```text
                                  shared LAN: 192.168.1.0/24
 client 192.168.1.10 ─────── gateway 192.168.1.2 ───── router 192.168.1.1
 default: 192.168.1.2          XRAT + engine               internet uplink
                                     └── proxy connection ────────> remote proxy
```

The gateway can receive client traffic and reach the router on the same physical
interface. The client and gateway must be able to communicate at layer 2; Wi-Fi
client isolation may prevent this. The gateway needs a stable LAN address and a
working physical route through the router. Selected clients must use that
address as their IPv4 default gateway. A router may be able to advertise it via
DHCP, but configuring or automating DHCP is outside the MVP. Clients retain
their connected LAN route; packets sent directly to another LAN device normally
never reach the gateway. Router-side policy routing is a different topology and
is not covered by this contract.

## Packet-flow contract

The following is an intended logical flow, not a description of verified kernel
hooks or engine settings. Routing and firewall details must be proven for each
engine, especially with ingress and physical egress on the same interface.

1. A client sends an off-LAN IPv4 packet to the gateway's LAN address at
   layer 2. Its IP destination remains the original remote destination.
2. Linux receives the packet as forwarded client traffic. Gateway-owned routing
   and firewall policy direct eligible traffic to the engine's managed TUN;
   unauthorized direct forwarding to the router is blocked.
3. The engine processes the captured flow according to its configuration and
   creates an outbound connection to the proxy endpoint. That connection is
   host-originated and must use the physical upstream route through the router,
   without re-entering capture. The remote proxy reaches the destination.
4. Return traffic reaches the engine through the proxy connection. The engine
   emits the response through the TUN and Linux routes it back to the client on
   the LAN. Tests must establish the actual return path and any necessary source
   translation; do not add NAT by default without evidence it is needed.

Host-originated traffic is a separate path: existing opt-in host TUN capture
continues to follow the [managed capture contract](managed-capture.md). Gateway
mode must not silently change host-only behavior when disabled. Management and
bootstrap traffic need an explicit physical path. XRAT must distinguish
host-originated engine connections from forwarded client packets even when both
use the same interface; an interface-only allow rule is insufficient.

The intended LAN policy preserves access to the gateway, router and explicitly
identified local subnets without proxying them. Direct same-subnet client-to-LAN
traffic bypasses the gateway by normal local routing; traffic that does reach
the gateway and matches an authorized LAN exception may follow the physical
route. Exceptions must be scoped to actual local destinations, not broad ranges
that accidentally permit direct internet forwarding. The exact exception policy
and return-path behavior remain to be validated for each backend.

## Routing, bootstrap and DNS

Before capture is activated, preflight must identify the LAN interface, its
address and subnet, the upstream route and proxy endpoint. Ambiguous or unsafe
topologies require an explicit selection or rejection before mutation. Gateway
forwarding requires IPv4 forwarding, but enabling it alone must never authorize
unprotected forwarding.

Proxy endpoint resolution and any bootstrap resolver must remain reachable
through a narrowly scoped physical path before and during capture. If an
endpoint is a hostname, account for DNS resolution, address changes, retries and
re-resolution; a stale one-time IP exception is not a complete design. Rules
must avoid capturing the engine's own outbound connection recursively and must
not create a general client bypass. Exact route, mark, interface binding and
resolver mechanisms are engine-specific decisions requiring tests. Fail safely
when a required bootstrap path cannot be established.

Client DNS is not automatically secured by choosing an IPv4 gateway. A DNS query
sent to an off-LAN IPv4 resolver can reach the gateway and must have a defined
UDP and TCP port 53 path. A query to a same-LAN resolver normally goes directly
to it; a client's own encrypted DNS, another gateway, or IPv6 can also take a
different path. The existing optional XRAT DNS listener is loopback-only; do not
assume LAN clients can use it. Define the supported resolver and interception
behavior, bootstrap dependencies and failure behavior in #300, and test them
independently for each engine. Do not claim all client DNS is protected merely
because port 53 is captured.

## Engine and platform boundary

| Target           | Gateway contract                             | Unresolved integration                                                                                                                                                                                                                      |
| ---------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Linux + sing-box | MVP target; unverified                       | Evaluate native TUN routing and `auto_redirect`, rule priorities, marks and nftables interaction in [#297](https://github.com/mhyrzt/xrat/issues/297).                                                                                      |
| Linux + Xray     | MVP target; unverified                       | Evaluate native TUN routing and outbound interface selection in [#305](https://github.com/mhyrzt/xrat/issues/305). Existing XRAT managed capture rejects Xray destination exclusions; do not assume sing-box's exclusion mechanism applies. |
| Linux + V2Ray    | Unsupported for this gateway MVP             | Separate future work.                                                                                                                                                                                                                       |
| macOS or Windows | Unsupported as a network gateway in this MVP | Existing host capture or machine-service support does not establish gateway support.                                                                                                                                                        |

The existing managed-capture version matrix and host-only tests do not establish
gateway capability, Linux ARM64 forwarding, or fail-closed protection. Supported
gateway engine versions and configuration must be established by native
preflight and independent acceptance for each engine. Reject unsupported or
unverified combinations before replacing a healthy session.

## Protection and ownership decisions

1. **Fail closed for traffic reaching the gateway.** While gateway protection is
   installed, forwarded client IPv4 internet traffic must not fall back to the
   physical upstream when the engine is unavailable, including startup,
   rotation, disconnect, crash, daemon restart and recovery. Authorized LAN and
   narrowly scoped host bootstrap/management traffic remain available. This is a
   requirement, not an existing guarantee. Traffic using a different client
   route, same-LAN DNS, or IPv6 that never reaches the gateway is outside its
   enforcement boundary.
2. **Install protection before enabling forwarding.** Persist intent and
   ownership before mutation; establish and verify the blocking policy before
   enabling a path that might forward client packets. Keep protection in place
   across runtime replacement and failure. On removal, prevent unprotected
   forwarding before removing the guard. Atomicity, ordering and reboot
   persistence must be demonstrated, including interrupted transitions.
3. **One runtime owner.** Gateway installation intent and protective resources
   are distinct from the managed runtime session. Reuse the existing runtime
   lifecycle and daemon supervisor rather than introducing a second proxy
   supervisor. A configured gateway, a live process, a TUN interface and proven
   packet forwarding are different facts; status must expose degraded or unknown
   states without treating one as proof of another.
4. **Own only identifiable resources.** Journal the identity and prior state of
   XRAT-managed firewall objects, routes, policy rules, interfaces and
   forwarding settings before changes. Detect conflicts and external
   modifications. Remove only verified owned resources; preserve unrelated rules
   and routes, and do not blindly restore a sysctl changed by another
   administrator. Retain the journal and report incomplete cleanup if identity
   or rollback is uncertain.
5. **Preserve host firewall policy.** Coordinate hook priorities, marks and
   engine-managed rules with existing host rules and the host kill switch. Do
   not flush or replace foreign firewall state. A host `OUTPUT` kill switch is
   not by itself protection for forwarded packets. Exact nftables/engine hook
   composition and conflict handling remain open pending implementation tests.

These decisions require
[preflight #296](https://github.com/mhyrzt/xrat/issues/296),
[routing #298](https://github.com/mhyrzt/xrat/issues/298),
[firewall #299](https://github.com/mhyrzt/xrat/issues/299),
[ownership #301](https://github.com/mhyrzt/xrat/issues/301), and
[lifecycle #302](https://github.com/mhyrzt/xrat/issues/302). Installation,
activation, removal and status interfaces are specified separately in
[#295](https://github.com/mhyrzt/xrat/issues/295).

## Packet-level acceptance

The implementation must extend the existing Linux namespace harness with
isolated client, gateway, router and controlled destination/proxy endpoints. The
client and router share the gateway's single physical LAN interface. Record
topology, engine versions, configuration, commands and observations. For
**each** engine independently, require:

- Forwarded TCP and UDP reach the intended destination through the controlled
  proxy; observe both destination and physical upstream, including return
  packets.
- LAN exceptions and management reachability work as documented, and unrelated
  LAN devices and foreign network state remain unaffected.
- Proxy outbound and DNS bootstrap use the physical upstream without recursion;
  test hostname resolution, endpoint changes and bootstrap failure.
- UDP and TCP DNS follow the documented path, including failure and bypass
  cases. Demonstrate the limits of same-LAN DNS and client IPv6 independently.
- With the proxy stopped or unreachable, and during connect, disconnect,
  rotation, crash and recovery, upstream observations show no direct client IPv4
  internet packets, including transition windows.
- Failed preflight and interrupted install/remove leave a safe, recoverable
  state; repeated operations preserve unrelated routes, firewall rules and
  forwarding settings. Test ownership conflicts and rollback failures.
- Exercise reboot-related persistence and recovery, clearly identifying what
  namespace tests simulate. Capture separate native Linux ARM64 results for each
  backend on the target Pi or equivalent hardware.

Configuration acceptance, passing unit tests, a live TUN, namespace results and
native hardware evidence are separate evidence levels. None alone proves all
gateway safety properties. See [#303](https://github.com/mhyrzt/xrat/issues/303)
for the implementation and acceptance work; user-facing deployment instructions
follow verified behavior in [#304](https://github.com/mhyrzt/xrat/issues/304).

## Linux packet processing and failure boundaries

The following describes normal IPv4 routing as a model, not the chosen engine
implementation. A received Ethernet frame is addressed to the gateway's MAC; its
IP source and destination remain the client's and the remote host's. In Linux,
ingress passes PREROUTING before the routing-policy database (RPDB) selects a
route. A non-local packet normally traverses FORWARD and POSTROUTING before
output; a packet addressed to a gateway-local address traverses INPUT. Locally
generated engine and bootstrap sockets take OUTPUT and POSTROUTING, not the
forwarded path. Engine redirection or DNAT/TPROXY can change the path, including
local delivery, so a FORWARD-only rule cannot be assumed sufficient until the
actual hooks and priorities are verified. An `accept` in one nftables base chain
does not override a later base-chain drop; a drop is decisive. Check all
relevant IPv4/inet hooks and priorities, including existing firewall, engine,
iptables-nft and flow-offload rules. Hardware or software flow offload that
bypasses the enforcement path is unsupported unless proven safe.

The RPDB evaluates rules by priority and may consult multiple tables; a main
route is not necessarily selected first. Existing Xray host capture generates
family-specific `/1` routes, while sing-box can manage its own table and rules.
More-specific LAN routes, rules and marks therefore need explicit precedence
analysis. A TUN receives IP packets written by kernel routing; packets injected
by the engine appear as ingress on that TUN and undergo another route lookup.
The exact engine capture and response path must be observed, not inferred from
an interface existing. TCP and UDP are proxy flows rather than a promise that
the original transport session crosses the remote proxy unchanged. Return
packets delivered to the client must have the expected remote source and client
destination; verify actual behavior separately for each engine and protocol.

No NAT is assumed necessary for proxied traffic: the engine terminates or
translates flows and creates its own upstream sockets. This is not proof of any
particular implementation. For an explicitly authorized routed LAN exception,
NAT is unnecessary only when the destination has a return route to the client; a
selected source NAT may be required if that cannot be arranged. NAT is never a
substitute for the forwarded-traffic guard or a reason to permit general direct
internet forwarding. Test both directions and preserve client identity whenever
possible. Same-interface forwarding may produce ICMP redirects or asymmetric
paths; prohibit redirects that would steer clients around the gateway and test
that no redirect or alternate route causes direct fallback.

Inspect `conf/all` and interface `rp_filter` and `src_valid_mark`: strict
reverse path checks can reject asymmetric TUN packets, and Linux uses the
maximum of the `all` and interface values. Select no blanket weakening: test
source validation with the actual policy and reject incompatible configurations,
or document a minimal journaled change with anti-spoofing and return-path tests.
Connection tracking, NAT, fragmentation, ICMP errors and path MTU also affect
packet handling. Existing established or offloaded connections must not bypass a
newly installed guard; test pre-existing TCP/UDP flows and fragments rather than
relying on an unconditional `ct state established,related accept`. IPv4
`ip_forward` changes can reset other IPv4 settings to host/router defaults;
snapshot affected settings, detect collateral changes, and restore only values
whose ownership and intervening changes can be established. If safe restoration
is not possible, refuse the operation and retain recovery information. See the
[Linux kernel IP sysctl documentation](https://www.kernel.org/doc/html/latest/networking/ip-sysctl.html).

### Traffic classes and trust

| Class                               | Required handling                                                                                                             |
| ----------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Selected client off-LAN IPv4        | Capture or drop; never direct physical internet forwarding.                                                                   |
| Local gateway management            | INPUT under the host's existing access policy; no implied open listener.                                                      |
| Connected LAN and router            | Direct only when destination and route are explicitly authorized; same-subnet traffic usually bypasses the gateway entirely.  |
| Off-subnet private networks         | Denied direct by default; exact configured destination prefixes and return path required. RFC1918 alone is not authorization. |
| Broadcast, multicast and link-local | Do not treat as generic internet exceptions; define narrowly scoped local handling or drop, and test.                         |
| Engine and bootstrap sockets        | Host-originated narrowly controlled upstream path, not an exemption for forwarded packets.                                    |

Client IP addresses, MAC addresses and packet marks are not authentication. An
on-link client can spoof source addresses; a privileged local process or other
networking component can set or change marks. Never accept forwarded traffic
solely because of a mark, destination matching the proxy endpoint, or physical
output interface. Mark bits and priorities need collision detection, masking and
trusted classification; exclude conflicting unknown rules. XRAT must define its
threat model and reject configurations where trustworthy classification cannot
be established. The guard must protect all client forwarding it enables,
including spoofed or unclassified ingress; client selection alone cannot be the
security boundary.

### Outbound path decision and open experiments

Prefer the existing engine-supported physical-interface selection and bounded
bootstrap policy where they can be proven to coexist with host capture. Compare
it with a separate policy-routing table and narrowly assigned socket marks. A
host route per endpoint is a weaker operational choice when DNS returns multiple
addresses or changes; it is acceptable only with atomic refresh and no gap or
broad client exemption. No candidate is selected as universally safe yet.

For each engine, #297 or #305 must run a controlled same-interface experiment
with literal and multi-address hostname endpoints, DNS answer changes and TTL,
re-resolution, route replacement, a restarted engine, a conflicting mark and
host-only capture enabled. Record `ip rule`, every relevant table, socket and
connection marks, nftables handles/counters, and captures at LAN, TUN, router
and proxy. Assert that each engine socket reaches only the intended upstream,
client packets to the same endpoint remain captured or dropped, and losing or
changing an upstream route fails closed. Establish whether binding survives
route changes and whether all engine-created and resolver sockets use it. Reject
an unproven combination rather than silently using a broad exception.

## Lifecycle and recovery contract

These states describe observed components, not a new runtime supervisor or a
single persisted truth. A status response distinguishes installation intent,
guard integrity, forwarding configuration, runtime/process health, TUN identity,
and last packet-test evidence; an observed ruleset cannot prove future packet
behavior. `protected` is asserted only if the installed guard and prerequisites
are verified; absent, externally changed, unobservable or conflicting state is
`unknown` or `degraded`, with an actionable reason. Packet success is a separate
observation.

| Transition                               | Required order and behavior                                                                                                                                                                                                                                                                                    |
| ---------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Preflight                                | Read-only discovery and native config validation; do not replace a healthy host-only session on failure.                                                                                                                                                                                                       |
| Install                                  | Persist durable intent, resource identities and before-values; install and read back a restrictive independent guard before adding capture routes or enabling forwarding. If forwarding already exists, protect it before declaring installation successful.                                                   |
| Activate/connect                         | Keep the guard active; establish required routes, start the existing runtime owner, verify owned TUN and state. Do not open direct forwarding during bootstrap or readiness failure.                                                                                                                           |
| Replace/rotate/disconnect                | Keep guard active throughout; unavailable or absent TUN means affected traffic drops. A deliberate disconnect does not remove the installed gateway guard.                                                                                                                                                     |
| Crash/restart                            | Kernel rules can outlive a process but are not inherently persistent across reboot. Preserve the guard; reconcile identity and intent before enabling capture or reporting protection.                                                                                                                         |
| Boot                                     | Require a platform integration that restores and verifies a restrictive guard before forwarding is enabled or traffic can use an already-enabled forwarding path. If ordering cannot be guaranteed, reject persistent gateway installation; a userspace daemon starting later is insufficient.                 |
| Remove                                   | Keep the guard until selected client forwarding is demonstrably disabled or otherwise blocked; restore only verified owned changes, then remove the guard. If pre-existing forwarding cannot be safely restored without creating a direct path, retain protection and require explicit administrator recovery. |
| Interrupted operation or external change | Stop progressing; retain durable journal and protection where still present, report degraded/unknown and reconcile only verified resources. If another privileged actor removes the guard, XRAT cannot guarantee fail-closed before detecting and repairing it.                                                |

A privileged administrator or another firewall manager can delete or override
resources; this is outside XRAT's enforcement guarantee. Detection and repair
are best effort and cannot erase the interval before detection. Installation
must not claim uninterrupted protection before the first guard is installed; if
forwarding already permits the traffic, report that pre-existing exposure.
Firewall transactions can atomically update one nftables ruleset, but cannot
atomically commit filesystem journals, sysctls, routes, engine processes and
boot ordering together. Use durable phase records and conservative
reconciliation at every boundary. Snapshot nftables family/table/chain/handle
and rule identity, route table/prefix/next hop/interface/metric, policy
priority/selector/action, interface index and identity, and before/applied
sysctl values. An intent record must survive a crash before mutation and only be
cleared after verified cleanup. A name or handle alone is insufficient proof
after object replacement. Repeat operations only when ownership and desired
state agree; never delete foreign or ambiguous objects, overwrite an
administrator's later changes, or erase the record of a failed rollback.
Distinguish XRAT-owned resources from engine-owned rules whose lifetime may end
with the engine.

## Preflight and engine integration decisions

Preflight is non-mutating, including dry-run. It validates Linux architecture,
engine executable/version and native schema, effective capabilities in both
owner and engine process, TUN availability, persistent firewall/boot ordering,
stable interface/address/subnet and neighbour reachability, selected client
scope, upstream next hop and route ambiguity. Check conflicts across RPDB
priorities/tables, marks and masks, nftables and iptables hooks, interfaces,
existing host kill switch, offload, sysctls and ownership journal. Check that
bootstrap has a viable nonrecursive route; a temporary or future reachability
failure is not proof of packet forwarding. Give the conflicting object and an
actionable diagnostic. Do not mutate a healthy host-only runtime to discover
whether gateway mode works.

The existing code offers useful building blocks, not gateway guarantees:
`runtime_service/tun_interface.rs` and `tun_ownership.rs` track TUN identity and
selected sing-box rules; `runtime_service/killswitch.rs` and `xrat-support`
firewall backends implement the host OUTPUT guard; runtime supervision owns
process transitions; `tests/runtime_network/` provides isolated engine fixtures.
The current host nftables rules include broad private-network and socket-mark
allowances and are not a forwarded-client firewall. Existing bootstrap DNS and
runtime generation are starting points, not evidence for forwarded clients.
Gateway rules require their own verified ownership and lifetime, coordinated
with rather than replacing those components.

| Backend  | Current host capture evidence and integration question                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| -------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| sing-box | XRAT's host contract requires >= 1.13.0 and pins 1.13.21 for acceptance. Its typed TUN generates `auto_route` and `route_exclude_address`, but does not currently generate `auto_redirect`. Upstream 1.13.21 documents Linux `auto_redirect` with `auto_route`, nftables, marks, table and rule indices; it conflicts with `route.default_mark` and dial `routing_mark`. Compare engine routing with and without it against the independent persistent guard; #297 must prove hook ordering, forwarded traffic and ownership before choosing. |
| Xray     | XRAT's host contract requires >= 26.7.11 and pins 26.7.11 and 26.9.30. XRAT emits `autoSystemRoutingTable` and `autoOutboundsInterface` for auto-route and rejects destination exclusions. #305 must compare engine auto-routes and XRAT-owned policy routes, prove LAN exceptions without unsupported fields, and confirm interface selection and loop prevention.                                                                                                                                                                           |

A further engine-routing boundary is essential: XRAT's Xray TUN builder adds a
broad private-address `direct` outbound rule, and user routing may select direct
outbounds for other destinations. Once the engine accepts a client flow, its
direct outbound is a host-originated socket; a forwarded-packet guard alone
cannot distinguish that socket from an authorized proxy connection. Gateway
configuration must reject or constrain direct and fallback engine routes for
client flows to the explicitly authorized LAN destinations, without silently
changing host-only behavior. Apply the same requirement to sing-box and to DNS
resolver detours. Test policy precedence, existing user rules, direct outbound
and proxy failure at the physical upstream. If flow origin cannot be reliably
distinguished, reject the combination rather than claim fail-closed protection.

These are host-capture version gates, not gateway support claims. Reject
versions, platforms, engine settings or conflicting firewall managers for which
the required combinations lack validation. V2Ray and non-Linux gateways remain
out of scope. Consult pinned
[sing-box 1.13.21 TUN documentation](https://github.com/SagerNet/sing-box/blob/v1.13.21/docs/configuration/inbound/tun.md)
and
[Xray 26.9.30 TUN configuration source](https://github.com/XTLS/Xray-core/blob/v26.9.30/infra/conf/tun.go);
upstream current documentation can describe newer behavior.

## DNS and IPv6 enforcement boundary

For off-LAN UDP/TCP 53 arriving at the gateway, #300 must choose and test a
specific interception or proxied resolver path, preserving query and response
semantics and specifying timeout, unavailable resolver and no-direct-fallback
behavior. No LAN listener is enabled by this specification. If later offered, it
must bind explicitly, restrict source access, define authentication and abuse
boundaries, and be independently tested; the present listener is loopback-only.
Encrypted DNS follows the route selected for its traffic but cannot be assumed
identifiable or universally enforced. Same-LAN resolver traffic normally never
reaches the gateway. IPv6 default routes, DNS and independently routed client
traffic can bypass an IPv4-only gateway entirely. Status must explain these
limits; correct IPv4 forwarding is not comprehensive device protection.

## Verification and traceability matrix

All gateway packet and native ARM64 results below are **pending**. The namespace
fixture in #303 must place client, gateway and router on one shared L2 segment
(e.g. a bridge plus veth endpoints); the gateway has exactly one LAN/upstream
physical-side interface and a separate TUN, not two physical-side NICs. A
controlled proxy and remote destination sit beyond the router. Capture at
client, gateway LAN/TUN and router egress; distinguish expected engine-to-proxy
packets from forbidden direct packets by addresses, ports and controlled
endpoint logs. Start capture before each transition and retain counters,
route/rule snapshots, engine/config versions, commands, timestamps and negative
observations.

| Requirement and decision                                  | Owner                  | Objective evidence                                                                                                                                                                                                   |
| --------------------------------------------------------- | ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Safe non-mutating discovery and host-session preservation | #295, #296             | Snapshot before/after dry-run and every rejection; existing session remains usable.                                                                                                                                  |
| Capture and return TCP/UDP, same interface                | #297, #298, #305, #303 | Proxy and destination logs plus TUN/LAN captures show expected flow and return; router sees no direct destination flow.                                                                                              |
| LAN/management and local exceptions                       | #298, #299, #303       | Controlled LAN endpoint and INPUT work; non-authorized private/public destinations do not bypass; spoofed sources and endpoint destinations cannot bypass.                                                           |
| Recursion and endpoint changes                            | #297, #305, #303       | Resolver, proxy and packet captures demonstrate multi-address refresh, upstream change, restart and collision behavior with no loop or client exemption.                                                             |
| Independent fail-closed guard and boot ordering           | #299, #301, #302, #303 | Continuous router capture and guard counters show no direct leak before readiness, during failures, rotation, interruption, recovery and actual reboot; externally removed guard is reported as unprotected/unknown. |
| DNS/IPv6 limits                                           | #300, #303, #304       | Controlled UDP/TCP resolver success/failure, same-LAN bypass and independent IPv6 observations; no claim about unreachable traffic.                                                                                  |
| Identity, rollback and preservation                       | #301, #303             | Inject failure at each durable phase; compare complete route, rule, firewall and sysctl snapshots, preserve foreign edits and retained journal on uncertain cleanup.                                                 |
| Engine and platform acceptance                            | #297, #305, #303, #304 | Pinned native config checks and independent namespace receipts for both engines, followed by separate ARM64 hardware receipts per engine.                                                                            |

Static review, native configuration validation, unit tests, namespace packet
runs and actual reboot/native hardware tests are distinct evidence levels.
Namespace destruction is not proof of reboot ordering or production recovery. No
gateway packet test or hardware acceptance was performed for this document.
