# SDK — Workspace And Reusable Library Surface

This folder collects work to turn the current single-crate application into a
modular Cargo workspace while preserving the installed product as one `xrat`
binary. The reusable proxy-management logic should live behind stable library
crates, and the CLI, TUI, and HTTP surfaces should become thin adapters over the
shared engine/runtime layers.

The crate already builds as a library (`src/lib.rs` re-exports every module as
`pub mod`), so the pure layers — `config` (link/subscription parsing), `xray` /
`singbox` (runtime config generation), `model`, `prober`, `support` — are
usable today. What is missing is a curated public SDK facade, crate boundaries
that keep frontend dependencies out of reusable logic, and a non-CLI way to
construct stateful engine/runtime services.

## Items

- `28-extract-reusable-sdk-crate.md` — Medium. Workspace architecture roadmap:
  split reusable config/model/prober/runtime/engine logic into library crates,
  expose a stable `xrat-sdk` facade, keep `xrat-cli`, `xrat-tui`, and
  `xrat-http` as adapters, and preserve one installed `xrat` binary.

## Dependencies

- `1-foundation/23-split-apperror-by-layer` — needed so the core can expose
  errors without leaking `reqwest`/`sqlx`/`toml`.
- `1-foundation/25-newtype-ids` — id types belong in the public API.
- `1-foundation/8-application-factories-test-setup` — the non-CLI `AppContext`
  constructor is the same seam test factories want.
- `2-use-cases/*` — shared use-cases become the SDK entry points for stateful
  (Tier 2) operations.
- `3-ports/*` — port traits let SDK consumers inject their own I/O.
