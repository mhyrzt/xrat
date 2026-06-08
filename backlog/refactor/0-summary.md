# Refactor Backlog Summary

The codebase has a useful split between CLI parsing, command handlers, daemon
supervision, Axum routes, TUI state, repositories, and runtime services, but the
application core is still incomplete. Several interfaces call repositories,
processes, daemon IPC, or formatting code directly, and some TUI and daemon
paths reuse CLI command modules instead of shared use-cases.

The highest-priority work is to extract shared application services with
explicit inputs/results, keep CLI/TUI/Axum/daemon adapters thin, and add test
seams around database, process, network, filesystem, and IPC dependencies.

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
