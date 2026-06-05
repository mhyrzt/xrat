# Backlog

Planned work, with live progress. Each item lists motivation, a proposed
direction, and a verifiable checklist. Check boxes as work lands so a future
session can resume mid-stream.

## Progress

| # | Item | Status |
|---|------|--------|
| 1 | Reconcile Removed Configs on Subscription Refresh | ✅ done |
| 2 | Move Import Ingestion Out of the TUI | ✅ done |
| 3 | Implement Automatic Subscription Refresh | ✅ done |
| 4 | Refresh Subscriptions Before Proxy Rotation | 🟡 in progress |

Legend: ⬜ not started · 🟡 in progress · ✅ done

## Themes

- **Ingestion ownership** — Move all new-source ingestion to the CLI and keep
  the TUI focused on operating already-stored sources.
- **Subscription freshness** — Make URL-backed subscriptions self-maintaining:
  reconcile on refresh, refresh on a schedule, and refresh before rotation.

## Dependency Order

1. Reconcile Removed Configs on Subscription Refresh — foundation; defines the
   reconciliation primitive (upsert present, soft-delete absent) reused by the
   rest.
2. Implement Automatic Subscription Refresh — depends on (1); schedules the same
   refresh + reconcile path.
3. Refresh Subscriptions Before Proxy Rotation — depends on (1) and reuses the
   refresh path from (2).

"Move Import Ingestion Out of the TUI" is independent of the freshness chain.

## Implementation Notes (discovered while mapping the code)

- Import flows converge on `Database::import_nodes(source, nodes)` →
  `repository::api::configs::import_nodes` → `subscriptions::find_or_create`
  (dedups only URL sources) + `configs::import_nodes` (the bulk upsert in
  `src/db/repository/configs/import_ops/import.rs`). Reconciliation belongs in
  that bulk upsert, keyed by `subscription_id`.
- `xrat import` and `xrat add` already exist (`src/cli/import.rs`,
  `src/cli/add.rs`, handlers in `src/app/commands/`). Item 2 is TUI removal +
  docs only; the CLI side is done.
- TUI source refresh currently lives in `src/tui/run/tasks/source.rs` via a
  private `import_from`. Item 3's scheduler should reuse a shared service, and
  the TUI should call it too.
- `subscriptions` table (migrations `0001_init.sql`, sqlite + postgres) has
  `source_url`, `source_kind`, `name`, `created_at`, `updated_at`. Item 3 needs
  a new `last_refreshed_at` column to make intervals survive daemon restarts.
- Rotation candidate selection: `resolve_replace_candidate_id` in
  `src/app/runtime_service/replace_flow/candidate.rs`. Non-manual triggers run
  `run_rotation_bulk_tests`. Refresh-before-rotation hooks in here.
- Config: `AppConfig` (`src/app/config/mod.rs`), runtime/rotation settings in
  `src/app/config/proxy/types.rs` + defaults in `proxy/default_values.rs`,
  defaults constants in `src/app/config/defaults.rs`, seed TOML in
  `src/app/commands/init_default_config.toml`.
- Events: `src/app/events.rs::record(...)`, sources include `SOURCE_ROTATION`.
  Add a subscription/refresh source + event kinds.

---

## 1. Reconcile Removed Configs on Subscription Refresh

**Problem.** Refreshing a subscription upserts the configs the provider returns
but preserves old configs that no longer appear in the payload. Provider-removed
configs accumulate as stale entries attached to the source.

**Decisions.**

- Reconcile by **soft-delete** (set `is_deleted`/`deleted_at`), not hard delete:
  reversible, avoids FK churn with `connection_tests`/`runtime_sessions`, and a
  later provider re-add is undone by the existing upsert (`is_deleted = FALSE`).
- Reconcile **unconditionally by `subscription_id`**. New subscriptions (file /
  raw_text get a fresh id each import) have no prior rows, so nothing is removed.
  This also answers the open question: manual `xrat import <url>` mapping to an
  existing subscription reconciles too, since it is semantically a refresh.
- **Skip reconcile when the parsed payload is empty** — defensive guard so a
  provider blip returning zero nodes cannot wipe an entire source.

**Checklist.**

- [x] Add `removed_configs: u64` to `ImportSummary` (`src/db/record/import.rs`).
- [x] In `configs::import_nodes`, after the upsert, soft-delete configs where
      `subscription_id = ? AND is_deleted = 0 AND dedup_key NOT IN (<new keys>)`;
      return the affected row count. Sqlite + Postgres branches.
      (`reconcile_removed` in `import_ops/import.rs`.)
- [x] Skip the soft-delete step when `nodes.is_empty()`.
- [x] Surface counts: `xrat import` output kv + TUI refresh status report removed.
- [x] Test (sqlite): import 2 nodes, re-import 1 (same URL) → `removed_configs`
      = 1, count drops, the absent config is soft-deleted, the present one stays.
      (`import_cases/reconcile.rs`.)
- [x] Test: empty re-import removes nothing; returning config is restored.
- [x] Postgres test (`postgres_cases::config_cases::verify_reconcile_state`,
      runs under `XRAT_POSTGRES_TEST_URL`).
- [x] `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`
      (412 passed).
- [x] Docs: `docs/src/02-cli/import.md` Behavior step describes reconciliation.

---

## 2. Move Import Ingestion Out of the TUI

**Problem.** The Sources tab exposes an import modal. It can import raw config
links, but doing so creates `raw_text` source rows that cannot be refreshed,
copied, or shared like real subscription sources, muddying the Sources model.

**Proposed direction.**

- Remove the TUI import modal and the `i` import key from the Sources tab.
- Keep source refresh actions in the TUI for already-known refreshable sources.
- Route all new ingestion through the existing CLI commands: `xrat import
  <input>` and `xrat add <link>`.
- Update TUI empty states, help text, and docs to point at the CLI commands.
- Restrict the Sources tab to inspect, rename, refresh, delete, copy/QR.

**Checklist.**

- [x] Remove `OpenImportModal` / `ImportInput` / `ImportBackspace` /
      `ImportSubmit` actions and `ImportModalState` (`src/tui/app/types.rs`,
      `mod.rs`, `defaults.rs`, `lifecycle.rs`).
- [x] Remove `render_import_modal`, the `i Import` help line
      (`src/tui/view/modals.rs`), and the `spawn_source_import` task
      (`src/tui/run/tasks/source.rs`). Dropped `import_modal_open` from the
      keymap signature and its threading in `run/mod.rs`.
- [x] Unbind the `i` key on the Sources tab (`src/tui/keymap/view.rs`).
- [x] Configs empty state already shows `xrat import` / `xrat add`
      (`view/configs/detail.rs`). Sources table always renders
      `All configs`/`Orphans` rows, so it has no empty placeholder to update.
- [x] Update `docs/src/02-cli/tui.md` to drop in-TUI import.
- [x] Update/remove affected keymap tests.
- [x] fmt + clippy + test (411 passed).

---

## 3. Implement Automatic Subscription Refresh

**Problem.** Subscriptions refresh only on manual TUI `r`/`R`. There is no
scheduler, so URL-backed subscriptions drift out of date without user action.

**Depends on:** item 1.

**Proposed direction.**

- New config section, e.g. `[subscriptions] auto_refresh = false`,
  `refresh_interval_hours = 24`. Add `SubscriptionSettings` to `AppConfig`,
  defaults, and seed TOML.
- Migration: add `subscriptions.last_refreshed_at TEXT` (sqlite + postgres) so
  intervals survive daemon restarts (scheduler picks rows whose
  `last_refreshed_at` is null or older than the interval).
- Shared refresh service in `src/app/` reused by the daemon scheduler and the
  TUI: list URL subscriptions, fetch + `import_nodes` (reconciles), stamp
  `last_refreshed_at`, record events.
- Daemon supervisor: refresh tick that runs due URL subscriptions when
  `auto_refresh` is enabled; skip non-URL sources.
- Record refresh start/success/failure via `events.rs`.

**Checklist.**

- [x] `SubscriptionSettings` + defaults + seed TOML + config tests
      (`config/subscriptions.rs`, `config/tests/sections.rs`).
- [x] Migration `0018_add_subscription_last_refreshed_at.sql` (both backends);
      `mark_refreshed` + `list_refreshable_due` repository fns, exposed on
      `Database`. `last_refreshed_at` stamped for URL sources inside
      `api::configs::import_nodes` (epoch-secs text, like `cooldown_until`).
- [x] Shared refresh service `src/app/subscription_refresh.rs`
      (`refresh_due` / `refresh_all`). Manual TUI/CLI import already stamps via
      `import_nodes`, so they share the same path.
- [x] Daemon scheduler: 300s detection ticker in the supervisor loop, guarded
      by an `AtomicBool`, spawns `refresh_due`. Interval honored per-subscription
      via persisted `last_refreshed_at` → survives restarts.
- [x] Events `subscription_refresh_started/succeeded/failed`
      (`SOURCE_SUBSCRIPTION`); per-sub failures swallowed, never crash daemon.
- [x] Docs: `docs/src/03-features/importing.md` "Refreshing Subscriptions".
- [x] Tests: due-selection + url-only filter
      (`import_cases/refresh_due.rs`); config parse/default/clamp tests.
- [x] fmt + clippy + test (415 passed).

Note: refresh interval is clamped to ≥1h (`refresh_interval_secs`) so a
misconfigured `0` cannot busy-loop the scheduler.

---

## 4. Refresh Subscriptions Before Proxy Rotation

**Problem.** Rotation selects a replacement from stored configs without first
refreshing URL-backed subscriptions, so it can pick stale or provider-removed
candidates.

**Depends on:** item 1; reuses the refresh service from item 3.

**Desired flow.** trigger → refresh URL subscriptions → reconcile → test
eligible enabled configs → pick lowest real-delay passing config → replace on
the same local inbound ports.

**Proposed direction.**

- Add `[runtime.rotation] refresh_subscriptions = true`.
- Refresh URL subscriptions before automatic timer/health candidate selection in
  `resolve_replace_candidate_id`; keep non-URL sources out.
- Manual `xrat proxy rotate`: add `--refresh` flag.
- Report refresh failures separately from candidate test failures.

**Open question.** Should manual rotation without `--config-id` run the same
fresh candidate test pass as automatic rotation instead of relying on persisted
results? (Defer; revisit during implementation.)

**Checklist.**

- [ ] `refresh_subscriptions` setting + default + seed TOML.
- [ ] Refresh hook in non-manual candidate selection; re-list after reconcile.
- [ ] `--refresh` flag on `xrat proxy rotate`.
- [ ] Separate refresh-failure events; old runtime not left stopped on failure.
- [ ] Docs: full rotation flow + manual/automatic differences.
- [ ] Tests.
- [ ] fmt + clippy + test.
