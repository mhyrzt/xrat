# Ports — Trait Seams For External Dependencies

This folder collects refactors that introduce trait-based ports around external
I/O (HTTP, processes, TCP, DNS, env, signals, platform, clipboard) so use-cases
become testable with fakes. Today only `GeoIpLookup` follows this pattern; every
other boundary is a concrete call.

Start with `13-trait-usage-gap-analysis.md` — it is the codebase-wide audit that
catalogs every direct-I/O seam and the rationale behind the individual port
items. `7-external-dependency-ports.md` is the high-level intro covering the
repository/daemon/clock/filesystem ports (a different set than the concrete ports
below).

## Items

- `13-trait-usage-gap-analysis.md` — Reference. Audit + index of all seams and
  affected files. Read this first.
- `7-external-dependency-ports.md` — Medium. Overview of `ConfigRepository`,
  `RuntimeProcessManager`, `InputReader`, `NetworkProbe`, `DaemonClient`, `Clock`,
  `Filesystem` ports.
- `14-http-client-port.md` — High. `HttpClient` trait over 8 direct `reqwest`
  call sites (upgrade, version-check, import, probers).
- `15-process-spawner-port.md` — High. `ProcessSpawner` over 10 direct
  `std::process::Command` sites (xray, sing-box, ping, systemctl, gsettings, …).
- `16-port-waiter-abstraction.md` — Medium. `wait_for_port` — unify the
  duplicated TCP-readiness polling loop in three engine startups.
- `17-dns-resolver-port.md` — Low. `DnsResolver` over `lookup_host` in TCP/ICMP
  probers.
- `18-local-ip-resolver-port.md` — Low. `primary_ip()` — de-duplicate the UDP
  local-IP trick.
- `19-signal-handler-port.md` — Low. `ctrl_c()` shutdown + signal sending.
- `20-platform-detector-port.md` — Low. `os()`/`arch()` over scattered
  `cfg!(target_os)`.
- `21-clipboard-port.md` — Low. `copy_text()` over `arboard` in TUI share.
- `22-env-vars-port.md` — Low. `get(key)` over scattered `std::env::var`.

## Dependencies

- `14` and `15` depend on `23-split-apperror-by-layer` (foundation): a port can
  only own its error type once `AppError` stops `#[from]`-ing `reqwest`.
- `15` and `16` are related — consolidate engine startup behind
  `ProcessSpawner` + `PortWaiter` (consider a unified `RuntimeProcessManager`).
- The `NetworkProbe`/`DnsResolver` ports back the test-execution use-case
  (`3-test-execution-use-case`).
