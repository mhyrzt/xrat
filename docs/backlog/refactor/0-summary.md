# Refactor Backlog Summary

The codebase has a useful split between CLI parsing, command handlers, daemon
supervision, Axum routes, TUI state, repositories, and runtime services, but the
application core is still incomplete. Several interfaces call repositories,
processes, daemon IPC, or formatting code directly, and some TUI and daemon
paths reuse CLI command modules instead of shared use-cases.

The highest-priority work is to extract shared application services with
explicit inputs/results, keep CLI/TUI/Axum/daemon adapters thin, and add test
seams around database, process, network, filesystem, and IPC dependencies.

The codebase uses only 4 custom traits (see `13-trait-usage-gap-analysis.md`).
`GeoIpLookup` is the only well-developed port pattern (with decorators, factory,
and test doubles). Zero repository/port/use-case abstraction traits exist — the
database layer, filesystem, IPC, process, and network calls are all concrete.
This makes unit tests depend on real I/O and discourages testing failure
scenarios.

## Items

- `01-config-query-use-cases.md`
- `02-config-lifecycle-service.md`
- `03-test-execution-use-case.md`
- `04-runtime-control-abstraction.md`
- `05-thin-daemon-supervisor-handlers.md`
- `06-centralized-dto-view-model-mapping.md`
- `07-external-dependency-ports.md`
- `08-application-factories-test-setup.md`
- `09-async-observability.md`
- `10-export-subscription-rendering.md`
- `11-tui-data-loading-boundaries.md`
- `12-pac-domain-module.md`
- `13-trait-usage-gap-analysis.md`
- `14-http-client-port.md`
- `15-process-spawner-port.md`
- `16-port-waiter-abstraction.md`
- `17-dns-resolver-port.md`
- `18-local-ip-resolver-port.md`
- `19-signal-handler-port.md`
- `20-platform-detector-port.md`
- `21-clipboard-port.md`
- `22-env-vars-port.md`
