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

**Problem:** The crate already builds as a library (`src/lib.rs` re-exports
every module as `pub mod`), so an external app can technically
`use xrat::config::...` today. But there is no curated public surface, no
crate-level separation between reusable proxy-management logic and frontends,
and no stable SDK facade. Three concrete blockers stop this from being a real
workspace architecture:

1. **Everything is `pub mod` with no curated API.** There is no public/internal
   boundary, so any internal refactor is a potential breaking change for an SDK
   consumer. There is no API contract to stabilize against.
2. **Heavy dependencies are always compiled.** `ratatui`, `axum`,
   `tonic`/`prost`, `sqlx`, `arboard` (clipboard), `crossterm`, `clap`, and
   `maxminddb` are unconditional. A consumer that only wants subscription-link
   parsing still pulls the TUI, HTTP server, gRPC, and database stacks.
3. **Stateful operations are welded to the CLI and a live database.**
   `AppContext` is
   `{ db: Database, app_config: AppConfig, runtime_paths: RuntimePaths }` and
   its only constructor is `AppContext::build(args: &cli::Cli)`. Anything under
   `src/app/` that takes `&AppContext` therefore requires a sqlx connection plus
   CLI-derived path resolution, so it cannot be used as a library without
   standing up a database and faking CLI args.

## Target Architecture

XRAT becomes a Cargo workspace whose installed product remains a single `xrat`
binary. Reusable proxy-management logic moves into stable library crates, while
frontend crates become adapters over shared engine and runtime crates.

| Crate                               | Role                                                                                                                                                                                                                                                                                                                                                |
| ----------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `xrat-model`                        | Shared domain types and newtype identifiers.                                                                                                                                                                                                                                                                                                        |
| `xrat-config`                       | Subscription import, protocol link parsing, normalization, runtime-neutral config types.                                                                                                                                                                                                                                                            |
| `xrat-prober`                       | TCP, ICMP, download, upload, real-delay probing.                                                                                                                                                                                                                                                                                                    |
| `xrat-db`                           | Database connection setup, records, repositories, migrations, persistence mapping.                                                                                                                                                                                                                                                                  |
| `xrat-runtime`                      | Xray/sing-box runtime config generation, process lifecycle, runtime sessions, reattach/replace/status, daemon-facing runtime control.                                                                                                                                                                                                               |
| `xrat-engine`                       | Interface-neutral use-cases and orchestration: import, list/detail, lifecycle, test, scan, connect/disconnect, status, logs.                                                                                                                                                                                                                        |
| `xrat-sdk`                          | Stable public facade for external Rust consumers. Curated proxy-management types and operations, no CLI/TUI/HTTP leakage.                                                                                                                                                                                                                           |
| `xrat-cli`, `xrat-tui`, `xrat-http` | Thin adapters: translate inputs, render outputs, call engine/runtime services.                                                                                                                                                                                                                                                                      |
| root crate (`xrat`)                 | Composition crate. Keeps both `[workspace]` and `[package]` in the root `Cargo.toml`: it is the binary **and** an implicit workspace member. Enables adapters and still installs the executable as `xrat`. (Decision: keep the existing root binary, not a separate `xrat-bin` — least churn, preserves `cargo install xrat` and `cargo run -- …`.) |

**Dependency direction (shared crates must not depend on frontend adapters):**

```text
root (xrat bin) -> xrat-cli + xrat-tui + xrat-http
xrat-cli/xrat-tui/xrat-http -> xrat-engine
xrat-sdk -> xrat-engine
xrat-engine -> xrat-runtime + xrat-config + xrat-db + xrat-prober + xrat-model
xrat-runtime -> xrat-config + xrat-db + xrat-model
```

**Why this change is needed:** Consumers (other Rust apps, future FFI/WASM
bindings, or internal frontends) need proxy domain logic — parse subscriptions,
generate Xray/sing-box runtime config, probe nodes, drive managed runtime
operations — without inheriting the entire CLI/TUI/server application. Without a
workspace split and curated SDK facade, every consumer pays full compile cost
and depends on unstable internal modules.

## Reusability Tiers (Extract Boundary)

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
- **Tier 3 — frontends, not SDK.** `src/cli`, `src/tui`, `src/server`. Entry
  points; pulling them yields the whole app, not a library.

## Prerequisites For Stateful Extraction

The pure Phase 1 crates do not require the application-layer prerequisites
below. These items must land before extracting database-backed runtime, engine,
SDK, or adapter behavior.

- [ ] `1-foundation/23-split-apperror-by-layer.md` — in progress: managed Xray
      and sing-box errors are layered; HTTP/config infrastructure conversions
      remain.
- [ ] `1-foundation/25-newtype-ids.md` — stable id types belong in the public
      API.
- [ ] `1-foundation/8-application-factories-test-setup.md` — the non-CLI
      `AppContext` constructor is the same seam test factories want.
- [ ] `2-use-cases/*` — business logic lives in shared use-cases rather than
      CLI/TUI/Axum adapters, so those use-cases become the SDK entry points for
      Tier 2.
- [ ] `3-ports/*` — port traits (HTTP, process, DNS, …) let SDK consumers inject
      their own I/O instead of the binary's concrete implementations.

## Implementation Plan

Stage it so each move preserves the binary and limits churn. **After every phase
the root binary must still build and release with all features enabled.** Move
one boundary at a time.

### Implemented first slice

- [ ] Convert the root manifest into a resolver-v3 workspace while retaining the
      root `xrat` package and binary.
- [ ] Extract `xrat-model`, `xrat-config`, and `xrat-prober`, configure them for
      ordered crates.io publication, and retain compatibility re-exports from
      the root crate.
- [ ] Keep managed runtime process control in the root; only the probe-specific
      Xray process wrapper belongs to `xrat-prober`.
- [ ] Fold decode helpers into `xrat-config`; keep the remaining mixed,
      application-coupled `support` modules in the root.
- [ ] Update workspace CI, Docker input, and crates.io publication order.
- [ ] Add programmatic `AppContext` construction and gate the root binary,
      frontends, database, runtime, clipboard, and GeoIP dependency stacks.

Phase 0 and Phase 1 are complete. Direct consumers can avoid frontend/database
dependencies by depending on the extracted crates or by building the root
library without default features.

### Next milestone

Complete the stateful-extraction prerequisites listed above. Once those seams
land, begin Phase 2 with `xrat-db`, verify both migration backends from its new
location, and then extract `xrat-runtime`.

> Global success gate, re-run at the end of every phase:
> `cargo build --workspace --locked`, `cargo test -q --workspace --locked`,
> `cargo clippy --locked --workspace --all-targets -- -D warnings`,
> `XRAT_PATH=<writable-temp-dir> cargo run --locked -- version`.

### Phase 0 — Prepare the single crate for extraction

Goal: make the current crate feature-gated and constructible without the CLI,
before any file moves.

- [ ] Add Cargo features for frontends and heavy deps: `cli`, `tui`, `server`,
      `clipboard`, `geoip`, plus database/runtime features. Default feature set
      must keep the binary identical to today.
- [ ] Gate `ratatui`, `axum`, `tonic`/`prost`, `arboard`, `crossterm`, `clap`,
      `maxminddb` behind the matching features (`optional = true` + feature
      wiring).
- [ ] Add a non-CLI engine/context constructor (e.g.
      `AppContext::new(db, app_config, runtime_paths)` or a builder) that takes
      already-resolved values instead of `&cli::Cli`.
- [ ] Reduce `AppContext::build(args: &cli::Cli)` to a thin adapter that
      resolves CLI args and calls the programmatic constructor.
- [ ] **Verify:** `cargo build --no-default-features` compiles the pure layers
      (`model`/`config`/`prober`/`support`) without TUI/server/db features.
- [ ] Add a test that constructs context without CLI args.
- [ ] **Verify:** `cargo build` (default features) and `cargo run -- version`
      remain unchanged after feature gating.

### Phase 1 — Extract pure/shared crates (lowest risk)

Goal: move stateless, dependency-light layers into workspace crates first.

- [ ] Create `Cargo.toml` workspace root with `members`; add `crates/` dir.
- [ ] Extract `xrat-model` (`src/model/` → `crates/xrat-model`). Update imports.
- [ ] Extract `xrat-config` (`src/config/` + `src/xray/` + `src/singbox/`
      parse + runtime-neutral config generation → `crates/xrat-config`). Depends
      on `xrat-model`.
- [ ] Extract `xrat-prober` (`src/prober/` → `crates/xrat-prober`). Depends on
      `xrat-model` and `xrat-config`.
- [ ] Move decode helpers into `xrat-config`; retain URL/time/GeoIP/platform and
      the other application-coupled support modules in the root. Do not add an
      `xrat-support` crate in this slice.
- [ ] **Verify:** each new crate builds standalone
      (`cargo build -p xrat-config`), unit tests move with their code and pass,
      root binary still builds.

### Phase 2 — Extract persistence and runtime crates

Goal: move stateful infra after the pure layers are stable.

- [ ] Extract `xrat-db` (`src/db/` records/repositories/migration wiring →
      `crates/xrat-db`). Keep `events` database/repository modules intact.
- [ ] Confirm both SQLite and Postgres migration paths still run from the new
      crate location.
- [ ] Extract `xrat-runtime` (Xray/sing-box runtime config generation + process/
      session lifecycle, reattach/replace/status, daemon-facing control →
      `crates/xrat-runtime`). Depends on `xrat-config` + `xrat-db` +
      `xrat-model`.
- [ ] **Verify:** runtime lifecycle tests pass; `cargo run -- connect`/`status`/
      `disconnect` smoke path works against a local config.

### Phase 3 — Create `xrat-engine`

Goal: interface-neutral application use-cases with typed request/result structs.

- [ ] Create `crates/xrat-engine`. Depends on `xrat-runtime` + `xrat-config` +
      `xrat-db` + `xrat-prober` + `xrat-model`.
- [ ] Move interface-neutral use-cases from command/TUI/server code into engine
      services (import, list/detail, lifecycle, test, scan, connect/disconnect,
      status, logs).
- [ ] Define typed request/result structs per use-case. No stdout printing, no
      CLI structs, no Axum DTOs inside the engine.
- [ ] CLI/TUI/HTTP/daemon code now translates inputs and renders outputs only —
      no business rules.
- [ ] **Verify:** engine use-cases have direct unit tests (not via CLI);
      adapters delegate to engine with no duplicated state-transition logic.

### Phase 4 — Create `xrat-sdk`

Goal: conservative, stable public facade.

- [ ] Create `crates/xrat-sdk`. Depends on `xrat-engine` (+ selected
      model/config/prober/runtime re-exports).
- [ ] Expose stable types only: `ProxyManager`, `ImportRequest`, `ImportResult`,
      `ConfigSummary`, `ConnectionStatus`, `TestOptions`, `TestResult`,
      `RuntimeStatus`.
- [ ] Do **not** expose: raw repository rows, CLI structs, Axum DTOs, process
      internals, sqlx/reqwest/toml error types.
- [ ] Add a `crates/xrat-sdk/examples/` example that parses a subscription,
      generates runtime config, and probes nodes using only the SDK.
- [ ] **Verify:** example builds and runs against `xrat-sdk` with default
      features only (no TUI/server pulled in).

### Phase 5 — Split adapters and binary composition

Goal: frontends become crates; binary becomes thin composition.

- [ ] Extract `xrat-cli` (`src/cli/` → `crates/xrat-cli`). Depends on
      `xrat-engine`.
- [ ] Extract `xrat-tui` (`src/tui/` → `crates/xrat-tui`). Depends on
      `xrat-engine`.
- [ ] Extract `xrat-http` (`src/server/` → `crates/xrat-http`). Depends on
      `xrat-engine`.
- [ ] Reduce the existing root crate to composition only: wire config, storage,
      runtime, adapters, tracing. Keep `[workspace]` + `[package]` in the root
      `Cargo.toml`; do **not** introduce a separate `xrat-bin`. Still
      builds/releases an executable named `xrat`.
- [ ] **Verify:** full feature build produces a single `xrat` binary identical
      in command surface; CLI parsing tests, TUI, and server routes all pass.

## Compatibility Constraints

The installed product, command names, release artifacts, Docker image,
systemd/launchd/rc.d templates, generated man pages, completions, desktop
assets, and `install.sh` flow must continue to target a single `xrat` binary.
The base workspace crates are published because the crates.io `xrat` package
depends on them. Their pre-1.0 APIs remain unstable; `xrat-sdk` stays private
until its facade is deliberately stabilized and released.

- [ ] Release workflow (`.github/workflows/release.yml`) still builds the `xrat`
      binary and its assets after the workspace split.
- [ ] `install.sh`, `Dockerfile`, packaging templates, man pages, and
      completions still resolve to `xrat`.

## Positive Effect On The Codebase

A consumer wanting only link parsing, config generation, probing, or managed
proxy operations can depend on a stable SDK instead of the full application. The
crate graph enforces frontend/core boundaries, reduces accidental coupling, and
makes adapter behavior easier to test against shared engine services.

## Risk / Migration Notes

- Do not start with a large file move if the use-case and port seams are not
  ready. Feature-gating, programmatic context construction, and use-case
  extraction (Phase 0 + prerequisites) must precede crate extraction for
  stateful code.
- Move one boundary at a time. Keep the root binary compiling with all features
  enabled after each phase.
- This is a deliberate roadmap, not a one-shot refactor. **Minimum viable
  slice:** parsing/config-generation/probing reuse is now available through
  Phase 1's `xrat-model`, `xrat-config`, and `xrat-prober`. Complete Phase 0's
  remaining stateful seams before extracting runtime, engine, or adapters.
