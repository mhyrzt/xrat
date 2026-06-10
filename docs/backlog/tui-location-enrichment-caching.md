# High, P1: TUI boot slow due to unconditional GeoIP location enrichment

### Status

Draft

### Scope

TUI startup latency. `TuiData::load` location enrichment, GeoIP/DNS resolution,
and where endpoint geo data is cached and read from.

### Symptom

TUI takes a long time to render its first frame. Regressed between v0.4.5 and
v0.4.6. Worse with more configs and with configs whose hostnames are slow or
unresolvable.

### Root cause

`TuiData::load` awaits `enrich_config_locations` before returning
(`src/tui/data/mod.rs:82`), so the first paint is blocked until every config's
location has been resolved.

Three compounding issues feed that blocking call:

1. Unconditional re-enrichment. `TuiConfigRow::needs_location_enrichment`
   hardcodes `return true` (`src/tui/data/configs.rs:111`, introduced in
   `7d3354a`). Every config is enriched on every boot, including rows that
   already carry `endpoint_country` / `endpoint_location` / `endpoint_asn` from
   their latest connection test. Those columns are already loaded from the
   database via `list_configs_with_latest_tests` (`LATEST_TEST_COLUMNS`,
   `lt_endpoint_*`), so the work is redundant.

2. DNS is the latency. `enrich_address` -> `resolve_address_ip` ->
   `tokio::net::lookup_host` (`src/support/geoip/enrich.rs:55`) runs per config
   with a 2s timeout (`ENRICH_TIMEOUT`) at concurrency 64
   (`ENRICH_CONCURRENCY`). The local mmdb lookup is fast; the DNS round trip is
   the cost. Many configs resolve in waves of 64, and each dead or slow host
   burns the full 2s before its wave can advance.

3. No persistent cache, despite one existing. `local_mmdb_lookup`
   (`src/tui/data/mod.rs:321`) builds a bare `LocalMmdbLookup` that is never
   wrapped in the existing in-memory `CachedLookup`
   (`src/support/geoip/cache/mod.rs`), and it is rebuilt on every `load()`. DNS
   results are not persisted anywhere. Endpoint geo is persisted only on
   `connection_tests` rows, so an untested config re-resolves on every boot
   forever, and a tested config re-resolves too because of issue 1.

### Possible solutions

- A. Gate enrichment. Make `needs_location_enrichment` return `false` when the
  row already has country/location/asn (and only attempt when an address is
  present). Removes redundant re-resolution of already-known configs. Smallest
  change; directly reverts the regression behavior.

- B. Non-blocking enrichment. Have `TuiData::load` return immediately with the
  geo already loaded from the database, and run enrichment in a background task
  that feeds results back into the TUI via the existing refresh path. First
  paint becomes instant; locations fill in shortly after.

- C. DB-persisted host -> geo cache. Add a cache table keyed by host (`host` PK,
  `ip`, `country`, `location`, `asn`, `resolved_at`) with a TTL. Boot reads the
  cache with zero network calls; a background pass refreshes stale or missing
  entries. Survives restarts and fixes both the untested-config case and
  repeated re-resolution of dead hosts. Requires a new ordered migration under
  `migrations/sqlite/` and `migrations/postgres/` and a small repository.

- D. In-session memory cache. Keep a persistent `CachedLookup` plus a DNS cache
  in TUI app state instead of rebuilding the lookup per `load()`. Speeds up
  in-session refreshes but does nothing for first boot.

### Recommendation

- Land A + B first: small, removes the regression and the blocking wait, gives
  an instant first paint.
- Add C as the durable fix so geo survives restarts and untested or unresolvable
  hosts stop re-resolving every launch.
- D is optional polish once a persistent lookup is in place.

### Changes required

- `src/tui/data/configs.rs`: real `needs_location_enrichment` predicate.
- `src/tui/data/mod.rs`: make enrichment non-blocking; stop rebuilding an
  uncached lookup per load; feed background results through the refresh path.
- For C: new migration + `geoip` cache repository in `src/db/`, read on load,
  write-through after resolution.

### Verification

- Measure time-to-first-frame with a large config set (e.g. hundreds), with and
  without unresolvable hosts; confirm first paint no longer waits on DNS.
- Unit test `needs_location_enrichment`: false when geo already present, true
  when address present and geo missing.
- Test that a config with persisted geo performs no DNS resolution on load.
- For C: repository test (SQLite and Postgres) for cache read/write and TTL
  expiry; test that a cached host resolves with no backend call.

---

## Related TUI performance / UX findings

Found during a broader TUI pass. Each is independent of the GeoIP work above but
compounds the same "TUI feels slow / freezes" symptom.

### 1. Mutating actions freeze the UI on an inline full reload (High)

Single config commands and bulk ops await a full `TuiData::load` directly in the
event loop, so the whole TUI is frozen (no redraw, no input) for the duration:

- `run/mod.rs:199` (`run_bulk_op`), `:234` (`run_source_rename`), `:237`
  (`run_source_delete`), `:240` (`run_clear_events`), `:261`
  (`run_config_command`) are all `.await`ed inline.
- `tasks/bulk.rs:18` and `tasks/commands.rs:34` call `reload_data` ->
  `TuiData::load`, which includes the slow geo enrichment waves.

So every enable/disable/delete/purge/restore pays the full reload + enrichment
cost synchronously while the UI hangs. A spawned variant already exists
(`spawn_reload_data`, `tasks/data.rs`). Fix: spawn the reload (feed result back
through the existing `task_tx` channel) instead of awaiting, or apply an
incremental in-memory update.

### 2. Full reload after a single-row change (Medium)

Enabling one config reloads all configs, sources, runtime, tests, and logs, then
re-enriches every row. `TuiApp::replace_config_row` (`app/lifecycle.rs:15`)
already supports incremental row update but mutation handlers do not use it.
Fix: mutate the affected row in place and reload fully only when the set of rows
actually changes (delete/purge/restore/import).

### 3. Unconditional ~10 Hz full re-render even when idle (Medium)

`event::poll(Duration::from_millis(100))` (`run/mod.rs:58`) returns on timeout
and falls through to `terminal.draw` every loop iteration. The widget tree is
rebuilt and log lines are re-wrapped ~10x/s even when nothing changed. ratatui
diffs the terminal writes, but the widget construction and formatting cost is
paid every frame. Fix: dirty-flag rendering — redraw only on input, task event,
log refresh, resize, or while an animation/spinner is active; otherwise skip the
draw.

### 4. Log wrapping recomputed every frame (Low)

`wrap_message` / `hard_break` (`view/configs/log.rs`) run per render for every
visible log row. Coupled with finding 3 this is per-frame CPU at idle. Fix: gate
behind dirty-flag rendering, or memoize wrapped output by (width, content).

### 5. First paint waits on data load + engine probe (Medium)

`run/mod.rs:21-23` awaits `TuiData::load` and then `probe_engines` (up to 2s per
engine) before the first `terminal.draw`. Combined with the enrichment delay,
the user sees nothing until all of it finishes. Fix: paint a skeleton frame
immediately, then fill configs, engines, and logs through the async task channel
already wired up.

### Suggested order

Finding 1 is the biggest perceived-freeze win and overlaps the enrichment fix
(both want a non-blocking `TuiData::load`). Do 1 + the enrichment B option
together, then 3 (idle CPU), then 2/5/4 as polish.
