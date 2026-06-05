# Backlog

## Review the new `validate` command

New, untracked: `src/cli/validate.rs` + `src/app/commands/validate.rs`
(`xrat validate <path>` — checks a `config.toml` exists, parses, and is
internally consistent). Needs a review pass before it's settled.

### Review points / smells spotted

- **Stage vocab drift:** `validate_runtime` allows rotation `test_stages` of
  `icmp | ping | real_delay | download`, but `ConnectionTestStage` only has
  `Icmp | RealDelay | Download` (no `ping`, no `upload`, no `tcp`). Confirm the
  accepted-string set matches what rotation/testing actually consume — `ping`
  may be a phantom and `upload` may be silently unvalidated.
- **Testing vs runtime mismatch:** `validate_testing` validates `tcp` and
  `download`/`real_delay`, but rotation stage list and `test_stage_name` don't
  agree on the same set. Single source of truth for stage names would prevent
  drift.
- **Hardcoded string matching** (`engine`, stage names, `network` tcp/udp)
  duplicates knowledge that lives in typed enums elsewhere — prefer parsing into
  the existing domain enums so validation can't fall out of sync.
- **Error aggregation** joins with `; ` into one `InvalidArgument` — fine, but
  consider structured multi-line output via `output.rs` helpers for readability.
- **Secret resolution** (`SecretString::resolve`) runs during validate — confirm
  validating shouldn't have side effects / shouldn't require env/file secrets to
  be present just to lint a config.

### Wiring to confirm

- Registered in `src/cli/mod.rs` + `src/app/commands/mod.rs` (both modified).
- Help text / `--help` output correct.
- Docs: add to `docs/src/02-cli/` (command reference). Not yet documented.
- Man page / completions regenerate to include it.
- Tests cover happy path + each error class (some unit tests present; consider a
  CLI parse test in `src/cli/tests/`).

### Open questions

- Should `validate` also be runnable against the _active_ config (no path arg,
  default to resolved config path)?
- Exit code convention on invalid (non-zero already via `AppError`)?
- `--format json` for machine-readable validation output?

## Docker-style short ref handles for configs/subscriptions

**Goal:** `xrat connect a1b2` (prefix match) instead of numeric ids.
Devops-native UX like `docker <id-prefix>`.

**Decision:** Keep `BIGSERIAL`/`INTEGER` primary keys as internal PKs. Do
**not** swap PKs to random strings (would force every FK to TEXT, rewrite all
child rows, lose monotonic insert order, bigger indexes). Instead add a separate
user-facing `ref` column.

### Approach

- Add `ref TEXT NOT NULL UNIQUE` to `configs` and `subscriptions`.
- `ref` = random 12-char hex generated on insert (not content-hash; stable
  across edits, no collision-meaning coupling). Revisit content-hash only if
  dedup signal wanted.
- User-facing lookup resolves by prefix:
  ```sql
  SELECT id, ref FROM configs WHERE ref LIKE 'a1b2%' AND deleted_at IS NULL;
  ```

  - 0 rows -> not found
  - 1 row -> resolve
  - > 1 rows -> ambiguous, ask for more chars
- Display `ref` (short form, e.g. first 8 chars) in list/show/status output;
  accept any unambiguous prefix on input.

### Changes required (future work)

- **Migrations** (`migrations/sqlite/` + `migrations/postgres/`): new ordered
  migration adding `ref` column + UNIQUE index to `configs` and `subscriptions`.
  Backfill existing rows with generated refs.
- **`src/db/record/`**: add `ref` field to config + subscription records.
- **`src/db/repository/`**: generate `ref` on insert; add prefix-resolve helper
  (`resolve_config_ref(prefix) -> id`, same for subscriptions) with
  ambiguous/not-found handling.
- **`src/db/database/`**: ensure UNIQUE index applied on both SQLite + Postgres.
- **`src/cli/`**: accept `<ref-prefix>` where commands take an id — `connect`,
  `show config`, `show subscription`, `parse`, `test`, `delete config`,
  `delete subscription`, etc. Decide: ref-only, or accept both numeric id and
  ref.
- **`src/app/commands/`**: route resolved ref -> internal id before existing
  logic; surface ambiguous-prefix and not-found errors.
- **`src/app/commands/output.rs`** + `--format` outputs (table/tsv/json/csv):
  add/replace id column with `ref` short form across list/show/status.
- **`src/tui/`**: show `ref` in lists/detail; any id-based selection/adapters.
- **`src/server/`**: API routes that key on config/subscription id — decide
  whether to expose `ref` and accept prefix in paths/queries.
- **`src/support/`**: shared ref-generation helper (random hex) + short-form
  truncation helper.
- **Tests**: ref generation uniqueness, prefix resolution (0/1/many), CLI
  parsing of ref args (`src/cli/tests/`), repository resolve on SQLite +
  Postgres.
- **`docs/src/02-cli/`**: document ref usage and prefix matching.

### Open questions

- Accept both numeric id **and** ref during transition, or ref-only?
- Apply to subscriptions too (recommended for consistent UX) or configs first?
- Random vs content-hash ref (default: random).

## BUG: `xrat upgrade` -> db migration error on next run

**Symptom:** after running `xrat upgrade`, got an error about db migrations.
(Exact error text not captured yet — need to reproduce and paste it.)

**Mechanism:** `xrat upgrade` only swaps the binary
(`src/app/commands/upgrade/mod.rs` -> `install_binary`). It does **not** run
migrations. Migrations run on the _next_ command invocation via
`src/db/schema.rs` (`SQLITE_MIGRATOR.run` / `POSTGRES_MIGRATOR.run`, sqlx
`migrate!`). So the failure surfaces on first use of the new binary, not during
upgrade itself.

**Most likely cause:** sqlx migration checksum/version mismatch. sqlx records a
checksum per applied migration in `_sqlx_migrations`. If any already-shipped
migration file was edited after release (content changed for a version a user
already applied), the new binary's embedded checksum != stored checksum ->
`VersionMismatch`. Editing past migrations in place is the classic trigger.
Other candidates: a new migration that fails against existing data, or a
partially-applied/dirty migration row.

### To investigate

- Reproduce: capture exact error string (likely
  `migration ... was previously applied but has been modified` or
  `VersionMismatch`).
- Audit `migrations/sqlite/` + `migrations/postgres/` git history: confirm no
  already-released migration file was edited in place. If yes, that's the bug —
  fix forward with a new migration, never edit applied ones.
- Check the two migrator trees are in sync (same count/ordering) so sqlite vs
  postgres don't diverge.

### Likely changes

- Policy: never edit shipped migrations; always add new ordered file.
- Better error surfacing: wrap `schema.rs` migrate errors with an actionable
  message (which migration, checksum mismatch, how to recover) instead of raw
  sqlx error.
- Consider running migrations as an explicit step during/after `upgrade` so the
  failure is tied to the upgrade action, not a later unrelated command.
- Possibly a `xrat db migrate` / repair path for dirty/mismatched state.
- Regression test: apply old schema, run current migrator, assert clean.

## TUI: redesign logs card into 3 tabs

Current `src/tui/view/configs/log.rs` only renders config `failure_reason`
lines. Replace with a tabbed logs card.

### Tabs

1. **xrat events** — internal app events only (session changed, ran tests,
   rotation, health, daemon, runtime transitions). Summary form, readable.
   Source: `src/app/events.rs` + `src/db/repository/events.rs` (same data as
   `xrat logs`).
2. **proxy engine** — raw logs from the proxy process (xray / sing-box). Source:
   process stdout/stderr / log file from `src/xray/process_mgmt/` (and sing-box
   equivalent). Need a tail/stream into the TUI.
3. **stats** — plots + text:
   - totals: total download, total upload
   - current delay / ping
   - failed request count
   - live graph of current traffic (throughput over time) Source: xray stats API
     (grpc/`StatsService`) or sing-box clash API. Need a poller feeding a ring
     buffer; ratatui sparkline/chart widget for the graph.

### Cross-cutting

- Logs + events rendered readable (trivial but explicit goal): aligned columns,
  level/kind coloring via `theme`, timestamps.
- Shortcuts: reset / clear per tab (e.g. clear visible buffer, reset stat
  counters). Wire into `src/tui/keymap/` + `chord.rs`; show in help/chrome.

### Changes required

- **`src/tui/view/configs/log.rs`**: rewrite as tab container + per-tab render.
- **`src/tui/app/types.rs`** (+ `src/tui/app/`): add active-tab state, stats
  ring buffer, proxy-log buffer.
- **`src/tui/data/`**: adapters to load xrat events, tail proxy logs, poll
  stats.
- **`src/tui/keymap/` + `chord.rs`**: tab-switch keys, reset/clear shortcuts,
  help entries.
- **`src/xray/process_mgmt/`** (+ sing-box): expose proxy log stream/path and a
  stats API client if not present.
- **proxy stats plumbing**: ensure generated runtime config enables the stats
  API (xray `stats`/`policy`/`api` inbound) so totals/traffic are queryable —
  check `src/xray/parsing/core/api.rs` / `policy.rs`.
- **Tests**: tab-switch state, keymap bindings (`src/tui/view/tests/` /
  `keymap`), stats buffer rollover.

### Open questions

- Stats source for sing-box vs xray — unify behind one trait or per-engine?
- Persist stats across reconnects, or reset each session?
- Proxy log: live tail (follow) vs snapshot on tab open?

## VHS tape demos in READMEs

Replace the static `media/screenshot.png` with animated VHS demos.

- Tapes already exist: `media/tapes/tui.tape` (-> `media/tui.gif`) and
  `media/tapes/cli.tape` (-> `media/cli.gif`).
- Render with `vhs media/tapes/tui.tape` + `vhs media/tapes/cli.tape`; commit
  the generated `media/tui.gif` + `media/cli.gif` (already untracked locally).
- Swap the screenshot `<img>` in `README.md` (currently `media/screenshot.png`,
  line ~24) for the gif(s).
- Same swap in `docs/src/README.md` (line ~16). Note docs uses
  `media/xrat-icon.png` path prefix — confirm the gif path resolves under mdBook
  build (likely needs `media/tui.gif` copied into the docs `src/media/` tree or
  referenced correctly).

### Changes required

- Generate + commit `media/cli.gif`, `media/tui.gif`.
- Edit `README.md` image block.
- Edit `docs/src/README.md` image block; verify mdBook asset path.
- Optional: CI step to regenerate tapes so demos don't go stale (vhs in a
  workflow), or a `just` recipe (`Justfile`) to rebuild gifs locally.

### Open questions

- One combined gif or two (cli + tui side by side / stacked)?
- Keep `screenshot.png` as fallback or drop it?
- Auto-regenerate in CI (heavier) vs manual refresh on release?

## `xrat daemon restart` subcommand

User changed `config.toml` and wants an easy restart to pick it up. Today only
`start | status | stop | install | uninstall` exist (`src/cli/daemon.rs`).

- Add `Restart(DaemonRestartArgs)` to the daemon `Subcommand` enum.
- Handler in `src/app/commands/daemon.rs`: stop then start. Reuse existing IPC
  paths (`ipc::daemon_shutdown_daemon` + the start flow); don't duplicate logic.
- Behavior: stop running daemon, reload config, start fresh. Decide whether it
  re-reads `config.toml` automatically (it should — that's the point) and what
  happens to the active runtime session (reattach vs clean restart).
- Edge cases: daemon not running -> just start (or error? pick one). systemd
  user-service case: prefer `systemctl --user restart xrat-daemon.service` when
  installed, vs raw IPC stop/start when run manually.

### Changes required

- `src/cli/daemon.rs`: new subcommand + args struct.
- `src/app/commands/daemon.rs`: restart dispatch arm.
- `src/cli/tests/cases/runtime_parse/daemon.rs`: parse test.
- Docs `docs/src/`: daemon section + man/completions regen.

### Open questions

- Plain stop+start, or a graceful in-place config reload (SIGHUP-style) that
  keeps the runtime session alive?
- When systemd-installed, delegate to `systemctl --user restart`?

## TUI: progress spinner during runtime switch

When runtime changes to a new config in TUI, it currently shows the `RuntimeOp`
task as plain "running" status text (`src/tui/run/tasks/runtime.rs`,
`src/tui/task/mod.rs` `TuiTaskKind::RuntimeOp`). Replace with a visible
in-progress indicator (spinner / animated marker) so the switch reads as active,
not frozen.

### Changes required

- `src/tui/task/`: track in-progress task kind + a tick/frame counter for
  spinner animation (advance on TUI tick).
- `src/tui/view/`: render spinner glyph next to the runtime status / in the
  runtime panel (`src/tui/view/configs/runtime.rs`) while `RuntimeOp` running.
- `src/tui/run/`: ensure the render loop ticks while a task is in flight so the
  spinner animates (may need a periodic redraw, not just input-driven).
- Clear spinner on completion/failure; show final state.
- Tests: in-progress state renders spinner, clears on done.

### Open questions

- Spinner only for `RuntimeOp`, or all blocking `TuiTaskKind`s (tests, etc.)?
- Braille/unicode spinner vs ascii — match nerd-font assumption already in
  tapes?
