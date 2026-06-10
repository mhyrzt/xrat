# Refactor Backlog Summary

The codebase has a useful split between CLI parsing, command handlers, daemon
supervision, Axum routes, TUI state, repositories, and runtime services, but the
application core is still incomplete. Several interfaces call repositories,
processes, daemon IPC, or formatting code directly, and some TUI and daemon
paths reuse CLI command modules instead of shared use-cases.

The highest-priority work is to extract shared application services with
explicit inputs/results, keep CLI/TUI/Axum/daemon adapters thin, and add test
seams around database, process, network, filesystem, and IPC dependencies.

The codebase uses only 4 custom traits (see `3-ports/13-trait-usage-gap-analysis.md`).
`GeoIpLookup` is the only well-developed port pattern (with decorators, factory,
and test doubles). Zero repository/port/use-case abstraction traits exist — the
database layer, filesystem, IPC, process, and network calls are all concrete.
This makes unit tests depend on real I/O and discourages testing failure
scenarios. Two further symptoms compound this: `AppError` is a 26-variant
god-enum that leaks `reqwest`/`sqlx`/`toml` coupling into every layer, and there
are ~123 production `unwrap()`/`expect()` panics that no test exercises.

## Folder layout

Items are grouped into three numbered folders that indicate the recommended
order of attack. Each folder has its own `summary.md`.

1. **`1-foundation/`** — cross-cutting structure and quality. Contains the
   prerequisites (error layering, newtype ids, shared test setup) that make the
   rest safer, plus observability, panic audit, e2e tests, and final file-split
   cleanup.
2. **`2-use-cases/`** — extract business logic from CLI/TUI/Axum/daemon adapters
   into shared application use-cases, services, and read models. The bulk of the
   High-priority core work.
3. **`3-ports/`** — trait seams around external I/O (HTTP, process, TCP, DNS,
   env, …) so use-cases become testable with fakes.

## Item numbering

File-number prefixes (`1`–`27`) are **stable identifiers**, not a global ranking,
and are cross-referenced between items (e.g. `15` references `16`, `13`
references `#1`–`#12`). Folder-number prefixes (`1`/`2`/`3`) indicate phase order.
Do not renumber files; use the order below to choose what to work on.

## Recommended order

### Phase 1 — foundation prerequisites

- `1-foundation/23-split-apperror-by-layer.md` — High. Unblocks HTTP/process port
  error ownership.
- `1-foundation/25-newtype-ids.md` — Medium. Before use-case/read-model
  signatures land.
- `1-foundation/8-application-factories-test-setup.md` — Medium. Shared fixtures
  for all later test work.

### Phase 2 — application core (High)

- `2-use-cases/1-config-query-use-cases.md`
- `2-use-cases/2-config-lifecycle-service.md`
- `2-use-cases/3-test-execution-use-case.md`
- `2-use-cases/4-runtime-control-abstraction.md`
- `2-use-cases/5-thin-daemon-supervisor-handlers.md`

### Phase 2 — ports that unblock and back the core (High)

- `3-ports/14-http-client-port.md`
- `3-ports/15-process-spawner-port.md`

### Phase 3 — read models, remaining ports, observability (Medium)

- `2-use-cases/6-centralized-dto-view-model-mapping.md`
- `2-use-cases/10-export-subscription-rendering.md`
- `2-use-cases/11-tui-data-loading-boundaries.md`
- `3-ports/7-external-dependency-ports.md`
- `3-ports/16-port-waiter-abstraction.md`
- `3-ports/17-dns-resolver-port.md`
- `1-foundation/9-async-observability.md`
- `1-foundation/24-audit-production-panics.md`
- `1-foundation/26-end-to-end-cli-tests.md`

### Phase 4 — narrow ports and final cleanup (Low)

- `2-use-cases/12-pac-domain-module.md`
- `3-ports/18-local-ip-resolver-port.md`
- `3-ports/19-signal-handler-port.md`
- `3-ports/20-platform-detector-port.md`
- `3-ports/21-clipboard-port.md`
- `3-ports/22-env-vars-port.md`
- `1-foundation/27-split-large-command-files.md` — verification step after the
  use-case extraction.

### Reference

- `3-ports/13-trait-usage-gap-analysis.md` — codebase-wide audit and rationale
  behind the port items.
