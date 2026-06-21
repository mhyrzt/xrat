# Use-Cases — Application Core Extraction

This folder collects refactors that pull business logic out of CLI/TUI/Axum/
daemon adapters into shared application use-cases, services, and read models. The
goal is one application core with thin adapters that only translate input and
render output.

Common theme: today the same behavior (config querying, lifecycle mutations,
test execution, runtime control, export rendering) is implemented separately per
interface, or one adapter reuses another adapter's module (TUI building
`cli::TestArgs`). These items define typed request/result structs and move the
rules into `src/app/use_cases/` or `src/app/services/`.

## Items

- `1-config-query-use-cases.md` — High. Shared config list/detail/export query
  service used by CLI, HTTP, and TUI.
- `2-config-lifecycle-service.md` — High. `ConfigLifecycleService` for enable/
  disable/delete/restore with typed outcomes instead of stdout printing.
- `3-test-execution-use-case.md` — High. Move bulk proxy-test pipeline into a
  use-case so TUI stops constructing `cli::TestArgs`.
- `4-runtime-control-abstraction.md` — High. `RuntimeControl` shared by CLI, TUI,
  and daemon over IPC vs local control.
- `5-thin-daemon-supervisor-handlers.md` — High. Extract runtime/rotation
  transition logic into services; keep supervisor handlers to channels + IPC
  mapping.
- `6-centralized-dto-view-model-mapping.md` — Medium. Application read models
  (`ConfigSummary`, `ConfigDetail`, …) mapped once, then to DTOs/rows at the edge.
- `10-export-subscription-rendering.md` — Medium. `ExportConfigsUseCase` /
  `PacFileUseCase` so `/json`, `/b64`, CLI, and PAC stop duplicating filters.
- `11-tui-data-loading-boundaries.md` — Medium. `DashboardService`/`OverviewUseCase`
  so `TuiData::load` stops doing direct DB/HTTP/process I/O.
- `12-pac-domain-module.md` — Low. Move PAC rendering/rules out of the Axum route
  module into a reusable proxy module.

## Dependencies

- `6` and `11` pair with `25-newtype-ids` (foundation) — adopt newtypes in new
  read-model signatures from the start.
- All benefit from `8-application-factories-test-setup` (foundation) for shared
  test fixtures.
- After extraction, large adapter files shrink — verified by
  `27-split-large-command-files` (foundation).
