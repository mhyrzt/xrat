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
scenarios. Two further symptoms compound this: `AppError` is a 26-variant
god-enum that leaks `reqwest`/`sqlx`/`toml` coupling into every layer, and there
are ~123 production `unwrap()`/`expect()` panics that no test exercises.

## Item numbering

File numbers (`01`–`27`) are **stable identifiers**, not a priority ranking, and
are cross-referenced between items (e.g. `15` references `16`, `13` references
`#1`–`#12`). Do not renumber files. Use the priority-ordered index below to
choose what to work on.

## Recommended order (by priority and dependency)

### High — application core extraction and the ports that unblock it

- `23-split-apperror-by-layer.md` — prerequisite: unblocks HTTP/process port
  error ownership (`14`, `15`)
- `01-config-query-use-cases.md`
- `02-config-lifecycle-service.md`
- `03-test-execution-use-case.md`
- `04-runtime-control-abstraction.md`
- `05-thin-daemon-supervisor-handlers.md`
- `14-http-client-port.md`
- `15-process-spawner-port.md`

### Medium — read models, ports, observability, and structural cleanup

- `25-newtype-ids.md` — do alongside `01`/`06` so new signatures adopt newtypes
  once
- `06-centralized-dto-view-model-mapping.md`
- `07-external-dependency-ports.md`
- `08-application-factories-test-setup.md`
- `26-end-to-end-cli-tests.md` — reuses `08` fixtures
- `09-async-observability.md`
- `10-export-subscription-rendering.md`
- `11-tui-data-loading-boundaries.md`
- `16-port-waiter-abstraction.md`
- `17-dns-resolver-port.md`
- `24-audit-production-panics.md` — pair per-module with the relevant use-case
  extraction; pairs with `23`
- `27-split-large-command-files.md` — verification step after `01`/`02`/`04`/`06`

### Low — narrow ports and isolated cleanups

- `12-pac-domain-module.md`
- `18-local-ip-resolver-port.md`
- `19-signal-handler-port.md`
- `20-platform-detector-port.md`
- `21-clipboard-port.md`
- `22-env-vars-port.md`

### Reference

- `13-trait-usage-gap-analysis.md` — codebase-wide audit and rationale behind the
  port items (`14`–`22`)

## All items (by number)

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
- `23-split-apperror-by-layer.md`
- `24-audit-production-panics.md`
- `25-newtype-ids.md`
- `26-end-to-end-cli-tests.md`
- `27-split-large-command-files.md`
