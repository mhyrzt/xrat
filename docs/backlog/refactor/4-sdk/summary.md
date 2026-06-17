# SDK — Reusable Library Surface

This folder collects work to turn the crate's reusable proxy domain logic into a
real, distributable SDK that another app can depend on without inheriting the
CLI/TUI/server frontends.

The crate already builds as a library (`src/lib.rs` re-exports every module as
`pub mod`), so the pure layers — `config` (link/subscription parsing), `xray` /
`singbox` (runtime config generation), `model`, `prober`, `support` — are
usable today. What is missing is a curated public API, feature-gated heavy
dependencies, and a non-CLI way to construct stateful context.

## Items

- `28-extract-reusable-sdk-crate.md` — Medium. Tiered reusability analysis
  (pure core vs `AppContext`/DB-bound vs frontends) and a staged plan:
  feature-gate frontends/heavy deps, add a non-CLI `AppContext` constructor,
  curate the public surface, then optionally split into an `xrat-core` workspace
  crate.

## Dependencies

- `1-foundation/23-split-apperror-by-layer` — needed so the core can expose
  errors without leaking `reqwest`/`sqlx`/`toml`.
- `1-foundation/25-newtype-ids` — id types belong in the public API.
- `1-foundation/8-application-factories-test-setup` — the non-CLI `AppContext`
  constructor is the same seam test factories want.
- `2-use-cases/*` — shared use-cases become the SDK entry points for stateful
  (Tier 2) operations.
- `3-ports/*` — port traits let SDK consumers inject their own I/O.
