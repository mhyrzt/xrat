---
id: TASK-44
title: Extract A Reusable SDK And Workspace Architecture
status: To Do
assignee: []
created_date: '2026-07-05 14:44'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-5
dependencies: []
priority: medium
ordinal: 28
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/4-sdk/28-extract-reusable-sdk-crate.md`

# Extract A Reusable SDK And Workspace Architecture

## Finding

### [Priority: Medium] Split reusable proxy management into workspace crates

**Files involved:**

- `src/lib.rs`
- `Cargo.toml`
- `src/config/` (parsing, normalization, protocol links, import)
- `src/xray/` and `src/singbox/` (parse + runtime config generation)
- `src/model/`
- `src/prober/`
- `src/support/`
- `src/app/context/mod.rs` (`AppContext::build(args: &cli::Cli)`)
- `src/app/` (stateful command/services layer)
- `src/cli/`, `src/tui/`, `src/server/` (frontends)
- future `crates/` workspace members

**Problem:** The crate already builds as a library (`src/lib.rs` re-exports every
module as `pub mod`), so an external app can technically `use xrat::config::...`
today. But there is no curated public surface, no crate-level separation between
reusable proxy-management logic and frontends, and no stable SDK facade. Three
concrete blockers stop this from being a real workspace architecture:

1. **Everything is `pub mod` with no curated API.** There is no public/internal
   boundary, so any internal refactor is a potential breaking change for an SDK
   consumer. There is no API contract to stabilize against.
2. **Heavy dependencies are always compiled.** `ratatui`, `axum`, `tonic`/`prost`,
   `sqlx`, `arboard` (clipboard), `crossterm`, `clap`, and `maxminddb` are
   unconditional. A consumer that only wants subscription-link parsing still
   pulls the TUI, HTTP server, gRPC, and database stacks.
3. **Stateful operations are welded to the CLI and a live database.**
   `AppContext` is `{ db: Database, app_config: AppConfig, runtime_paths:
   RuntimePaths }` and its only constructor is
   `AppContext::build(args: &cli::Cli)`. Anything under `src/app/` that takes
   `&AppContext` therefore requires a sqlx connection plus CLI-derived path
   resolution, so it cannot be used as a library without standing up a database
   and faking CLI args.

**Target architecture:** XRAT should become a Cargo workspace whose installed
product remains a single `xrat` binary. Reusable proxy-management logic moves
into stable library crates, while frontend crates become adapters over shared
engine and runtime crates:

- `xrat-sdk` — stable public facade for external Rust consumers. Exposes curated
  proxy-management types and operations without leaking CLI/TUI/HTTP details.
- `xrat-engine` — interface-neutral use-cases and orchestration for import,
  list/detail, lifecycle, test, scan, connect/disconnect, status, and logs.
- `xrat-runtime` — Xray/sing-box runtime config generation, process lifecycle,
  runtime sessions, reattach/replace/status flows, and daemon-facing runtime
  control.
- `xrat-config` — subscription import, protocol link parsing, normalization, and
  runtime-neutral config types.
- `xrat-db` — database connection setup, records, repositories, migrations, and
  persistence mapping.
- `xrat-prober` — TCP, ICMP, download, upload, and real-delay probing.
- `xrat-model` — shared domain types and newtype identifiers.
- `xrat-cli`, `xrat-tui`, `xrat-http` — thin adapters that translate inputs,
  render outputs, and call engine/runtime services.
- `xrat-bin` or the existing root binary — composition crate that enables the
  adapters and still installs the executable as `xrat`.

**Dependency direction:** Shared crates must not depend on frontend adapters.
The intended direction is:

```text
xrat-bin -> xrat-cli + xrat-tui + xrat-http
xrat-cli/xrat-tui/xrat-http -> xrat-engine
xrat-sdk -> xrat-engine
xrat-engine -> xrat-runtime + xrat-config + xrat-db + xrat-prober + xrat-model
xrat-runtime -> xrat-config + xrat-db + xrat-model
```

**Why this change is needed:** Consumers (other Rust apps, future FFI/WASM
bindings, or internal frontends) need proxy domain logic — parse subscriptions,
generate Xray/sing-box runtime config, probe nodes, and drive managed runtime
operations — without inheriting the entire CLI/TUI/server application. Without a
workspace split and curated SDK facade, every consumer pays full compile cost
and depends on unstable internal modules.

**Current reusability tiers (extract boundary):**

- **Tier 1 — already SDK-ready, stateless, no `AppContext`/DB/filesystem:**
  - `xrat::config` — `parse_text`, `parse_link`, `parse_links_batch`,
    `parse_link_with_engine`, `parse_import` (subscription/protocol links →
    `Node`).
  - `xrat::xray` / `xrat::singbox` — parse + runtime config generation.
  - `xrat::model` — shared domain types (`Node`, …).
  - `xrat::prober` — TCP/ICMP/download/upload/real-delay test runners.
  - `xrat::support` — decode, GeoIP, URL, network, time helpers.
- **Tier 2 — usable but drags weight.** Everything under `src/app/`. Functions
  take `&AppContext`, forcing a `Database` (sqlx, sqlite/postgres), `AppConfig`,
  and `RuntimePaths`, and the only constructor couples to `cli::Cli`.
- **Tier 3 — frontends, not SDK.** `src/cli`, `src/tui`, `src/server`. These are
  entry points; pulling them yields the whole app, not a library.

**How to implement it:** Stage it so each move preserves the binary and limits
churn.

1. **Prepare the single crate for extraction.** Feature-gate frontends/heavy deps
   (`cli`, `tui`, `server`, `clipboard`, `geoip`, database/runtime features) and
   add a non-CLI `AppContext` or engine context constructor. Keep
   `AppContext::build(args: &cli::Cli)` as an adapter over the programmatic
   constructor.
2. **Extract pure/shared crates first.** Move `model`, `config`, and `prober`
   into workspace crates before stateful runtime/database code. These are the
   lowest-risk boundaries because they already have relatively clear inputs and
   outputs.
3. **Extract persistence and runtime crates.** Move database records/
   repositories/migration wiring into `xrat-db`, and move Xray/sing-box runtime
   config generation plus process/session lifecycle into `xrat-runtime`.
4. **Create `xrat-engine`.** Move interface-neutral application use-cases from
   command/TUI/server code into engine services with typed request/result
   structs. CLI/TUI/HTTP/daemon code should translate inputs and render outputs,
   not own business rules.
5. **Create `xrat-sdk`.** Expose a conservative public facade over selected
   model/config/prober/engine/runtime APIs. Prefer stable types such as
   `ProxyManager`, `ImportRequest`, `ImportResult`, `ConfigSummary`,
   `ConnectionStatus`, `TestOptions`, `TestResult`, and `RuntimeStatus`; do not
   expose raw repository rows, CLI structs, Axum DTOs, or process internals.
6. **Split adapters and binary composition.** Move frontend code into
   `xrat-cli`, `xrat-tui`, and `xrat-http` crates. Keep the final binary as a
   small composition layer that wires config, storage, runtime, adapters, and
   tracing, and still builds/releases an executable named `xrat`.

**Compatibility constraints:** The installed product, command names, release
artifacts, Docker image, systemd/launchd/rc.d templates, generated man pages,
completions, desktop assets, and `install.sh` flow must continue to target a
single `xrat` binary. Workspace crate names are internal packaging details unless
and until `xrat-sdk` is published as a public crate.

**Positive effect on the codebase:** A consumer wanting only link parsing,
config generation, probing, or managed proxy operations can depend on a stable
SDK instead of the full application. The crate graph enforces frontend/core
boundaries, reduces accidental coupling, and makes adapter behavior easier to
test against shared engine services.

**Risk / migration notes:** Do not start with a large file move if the use-case
and port seams are not ready. Feature-gating, programmatic context construction,
and use-case extraction should precede crate extraction for stateful code. Move
one boundary at a time and keep the root binary compiling with all features
enabled after each step. This item depends on and reinforces:

- `1-foundation/23-split-apperror-by-layer.md` — a layered error type is needed
  before the core can expose errors without leaking `reqwest`/`sqlx`/`toml`.
- `1-foundation/25-newtype-ids.md` — stable id types belong in the public API.
- `1-foundation/8-application-factories-test-setup.md` — the non-CLI
  `AppContext` constructor is the same seam test factories want.
- `2-use-cases/*` — once business logic lives in shared use-cases rather than
  CLI/TUI/Axum adapters, those use-cases become the natural SDK entry points for
  Tier 2.
- `3-ports/*` — port traits (HTTP, process, DNS, …) let SDK consumers inject
  their own I/O instead of the binary's concrete implementations.

Recommendation: keep this as a deliberate roadmap, not a one-shot refactor. If
only parsing/config-generation/probing reuse is needed first, extract
`xrat-model`, `xrat-config`, and `xrat-prober` before moving the stateful engine
and adapter crates.
<!-- SECTION:DESCRIPTION:END -->
