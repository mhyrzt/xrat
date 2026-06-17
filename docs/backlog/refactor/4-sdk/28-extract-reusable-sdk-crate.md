# Extract A Reusable SDK / Library Crate

## Finding

### [Priority: Medium] Carve a curated `xrat-core` SDK out of the binary crate

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

**Problem:** The crate already builds as a library (`src/lib.rs` re-exports every
module as `pub mod`), so an external app can technically `use xrat::config::...`
today. But there is no curated public surface and no separation between the pure,
reusable core and the application/frontend layers. Three concrete blockers stop
this from being a real SDK:

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

**Why this change is needed:** Consumers (other Rust apps, a future FFI/WASM
binding, or internal reuse) want the proxy domain logic — parse a subscription,
generate an Xray/sing-box runtime config, probe a node — without inheriting the
entire CLI/TUI/server application. Without a curated, feature-gated core, every
such consumer pays full compile cost and rides an unstable, undocumented API.

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

**How to implement it:** Stage it, smallest-risk first.

1. **Feature-gate the frontends and heavy deps.** Add cargo features (e.g.
   `tui`, `server`, `cli`, `clipboard`, `geoip`) that gate `src/tui`,
   `src/server`, `src/cli`, `arboard`, and `maxminddb`. Make `ratatui`,
   `crossterm`, `axum`, `tonic`/`prost`, `clap*` optional and tied to those
   features. The binary enables them all; a library consumer enables only what
   it needs. This alone makes Tier 1 cheap to depend on without restructuring.
2. **Add a non-CLI `AppContext` constructor.** Introduce a plain config/builder
   input (no `cli::Cli`) so `AppContext` can be created programmatically. Keep
   `build(args: &cli::Cli)` as a thin adapter over it. Unblocks Tier 2 library
   use and pairs with `1-foundation/8-application-factories-test-setup.md`.
3. **Curate the public surface.** Replace blanket `pub mod` in `src/lib.rs` with
   an explicit `pub use` facade (e.g. an `sdk` or `prelude` module) that
   re-exports the Tier 1 entry points and the domain types, and make internals
   `pub(crate)` or `#[doc(hidden)]`. This is the API contract to version against.
4. **(Optional, larger) Split into a workspace.** `xrat-core` (Tier 1 + model,
   no frontends) plus `xrat` (binary depending on core, owning `cli`/`tui`/
   `server`/`app`). Cleanest long-term boundary; do it only after steps 1–3
   prove the seam.

**Positive effect on the codebase:** A consumer wanting only link parsing or
config generation compiles a small dependency tree against a documented, stable
API. The pure/stateful boundary becomes explicit and enforced by the crate
graph, which also pays back into testability and faster builds for the core.

**Suggested target architecture:** `xrat-core` exposes pure domain capabilities
(parse, normalize, generate runtime config, probe) with a curated facade and no
frontend deps; the `xrat` binary keeps CLI/TUI/server/daemon and the stateful
`AppContext`/`Database` orchestration on top of core.

**Risk / migration notes:** Steps 1–3 are incremental and non-breaking for the
binary if the binary enables all features by default. The workspace split (step
4) is the only large move and should follow, not precede, the feature-gating and
API curation. This item depends on and reinforces:

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

Recommendation: if only parsing / config-generation / probing reuse is needed,
Tier 1 is usable now and step 1 (feature-gating) is the highest-value, lowest-
risk work. Defer the workspace split until a real Tier 2 consumer exists.
