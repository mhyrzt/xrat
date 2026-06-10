# Foundation — Cross-Cutting Structure And Quality

This folder collects cross-cutting refactors that are not tied to one feature but
make the rest of the backlog safer and easier: error-type structure, type-safe
ids, test infrastructure, observability, panic reduction, and file-size cleanup.

Several items here are **prerequisites** for the `use-cases/` and `ports/` work
and should be sequenced early; two are **verification/cleanup** steps that come
last.

## Items

- `23-split-apperror-by-layer.md` — High, **prerequisite**. Break the 26-variant
  `AppError` god-enum; stop leaking `reqwest`/`sqlx`/`toml` `#[from]` into every
  layer. Unblocks HTTP/process port error ownership (`14`, `15` in ports).
- `25-newtype-ids.md` — Medium, **prerequisite**. `ConfigId`/`SubscriptionId`/
  `ConfigRef` newtypes. Adopt in `1`/`6` (use-cases) signatures from the start.
- `8-application-factories-test-setup.md` — Medium, **prerequisite**.
  `TestAppBuilder`/`TestContext`/production factories; de-duplicates test setup
  reused by everything else, including `26`.
- `9-async-observability.md` — Medium. Structured tracing for swallowed
  best-effort async failures in daemon/IPC/TUI paths.
- `24-audit-production-panics.md` — Medium. Triage ~123 production `unwrap()`/
  `expect()` calls. Pair per-module with the use-case extraction; pairs with
  `23`.
- `26-end-to-end-cli-tests.md` — Medium. Black-box `assert_cmd` tests over a temp
  home; behavior-preserving harness for the larger refactors. Reuses `8`.
- `27-split-large-command-files.md` — Medium, **verification step**. Confirm the
  oversized command files (`list.rs` 724L, `rotate.rs` 617L, …) actually shrank
  after `1`/`2`/`4`/`6` (use-cases) moved logic out. Do last, not standalone.

## Suggested local order

1. `23` (error layering) — unblocks ports.
2. `25` (newtypes) — before use-case/read-model signatures land.
3. `8` (test setup) — shared fixtures for all later test work.
4. `9`, `24`, `26` — observability, panic audit, e2e tests (alongside the feature
   work they cover).
5. `27` — final cleanup once logic has moved.
