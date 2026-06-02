# Phase 8: GeoIP MMDB Assets and Lookup Backends

## Goal

Finish XRAT's GeoIP work as one coherent phase with two closely related parts:

- local GeoLite2 MMDB asset management through `xrat geoip ...`
- pluggable lookup backends for test enrichment (`mmdb`, remote services, and
  later fallback chains)

The combined phase should leave XRAT with:

- a dedicated MMDB asset directory under `XRAT_PATH` / `~/.config/xrat`
- top-level MMDB asset configuration under `[mmdb]`
- read-only GeoIP CLI commands (`path`, `status`) plus downloader commands
- local MMDB lookup remaining the default backend
- remote lookup backends configurable from `config.toml`
- a clean separation between:
  - `[mmdb]` for MaxMind asset management
  - `[testing.geoip]` for lookup behavior
  - `[geo]` for routing geo assets such as `geosite.dat` / Xray `geoip.dat`

## Final Architecture

### Config split

```toml
[mmdb]
dir = "mmdb"
download_url = "https://github.com/P3TERX/GeoLite.mmdb/releases/latest/download/{edition}.mmdb"
timeout_secs = 60
default_editions = ["country", "city", "asn"]
auto_update = false
update_interval_hours = 168

[testing.geoip]
enabled = true
backend = "mmdb"
fallback = "none"
country_path = "mmdb/GeoLite2-Country.mmdb"
city_path = "mmdb/GeoLite2-City.mmdb"
asn_path = "mmdb/GeoLite2-ASN.mmdb"

[testing.geoip.remote]
provider = "ipwhois"
endpoint = ""
timeout_ms = 5000
api_key = ""
rate_limit_per_minute = 30

[testing.geoip.cache]
enabled = true
ttl_secs = 86400
max_entries = 10000
```

### Directory layout

- `~/.config/xrat/mmdb/GeoLite2-Country.mmdb`
- `~/.config/xrat/mmdb/GeoLite2-City.mmdb`
- `~/.config/xrat/mmdb/GeoLite2-ASN.mmdb`
- `~/.config/xrat/geo/...` for routing geo assets only

### Why one merged plan now

The downloader work and remote-backend work are no longer separate enough to be
planned independently because they now share:

- the `[mmdb]` config model
- the MMDB path resolver
- the `xrat geoip` command family
- the same end-user GeoIP story

## Current Progress

### Progress report

- The local MMDB asset-management path is now established end-to-end in Rust:
  config, resolver, CLI scaffold, downloader core, and update alias all exist.
- The old monolithic `src/support/geoip.rs` has been split into a module tree
  with a shared trait, local MMDB adapter, backend builder, and the first remote
  backend (`ipwhois`).
- `ResolvedTestSettings` already carries `geoip_lookup`, but the prober still
  uses the old sync call path; the async switchover is still pending.
- The next highest-value slice is remote hardening: cache and rate limiting,
  then the second backend (`ip-api`), then async prober integration.

Implemented already:

- [x] Added top-level `[mmdb]` config in `src/app/config/mmdb.rs`
- [x] Wired `AppConfig.mmdb`
- [x] Added MMDB defaults in `src/app/config/defaults.rs`
- [x] Switched default testing GeoIP paths from `geoip/...` to `mmdb/...`
- [x] Added shared MMDB path resolution in `src/app/paths/mmdb.rs`
- [x] Made built-in MMDB defaults resolve from `RuntimePaths.root_dir`
- [x] Preserved explicit custom relative MMDB paths as config-relative
- [x] Added `xrat geoip path`
- [x] Added `xrat geoip status`
- [x] Added `xrat geoip download` CLI shape and core downloader implementation
- [x] Added `xrat geoip update` CLI shape as `download --all --force`
- [x] Added MMDB edition parsing for canonical and short names
- [x] Added `AppError::GeoipDownload`
- [x] Split `src/support/geoip.rs` into `src/support/geoip/`
- [x] Added `GeoIpLookup` and `GeoIpError`
- [x] Added `LocalMmdbLookup` with preserved free-function wrappers
- [x] Moved endpoint classification into shared GeoIP support code
- [x] Extended typed `[testing.geoip]` config with backend, fallback, remote,
  and cache settings
- [x] Added the first lookup-chain builder around `LocalMmdbLookup`
- [x] Added the first remote backend: `RemoteIpWhoisLookup`
- [x] Wired `backend = "ipwhois"` through the lookup builder
- [x] Added `geoip_lookup` to resolved test settings without changing prober
  behavior yet
- [x] Added CLI parsing tests for the new `geoip` command family
- [x] Updated example config with `[mmdb]`

Not implemented yet:

- [ ] downloader integration tests with a local HTTP fixture
- [ ] multi-edition concurrency and richer summary handling
- [ ] remaining remote lookup backends under `src/support/geoip/`
- [ ] `ip-api` backend
- [ ] cache and rate-limit decorators
- [ ] async prober integration
- [ ] `xrat geoip lookup`
- [ ] `xrat geoip backend`

## Current Code Shape

Relevant code already in place:

- `src/app/config/mmdb.rs`
- `src/app/paths/mmdb.rs`
- `src/cli/geoip.rs`
- `src/app/commands/geoip/path.rs`
- `src/app/commands/geoip/status.rs`
- `src/app/commands/test/settings/resolve.rs`

The current shell script still downloads into `${XRAT_PATH}/geoip`, but the new
Rust direction intentionally moves the default location to `${XRAT_PATH}/mmdb`.

## CLI Plan

### Local asset management

```bash
xrat geoip download [flags]
xrat geoip update
xrat geoip path
xrat geoip status
```

### Lookup inspection

```bash
xrat geoip lookup <ip>
xrat geoip backend
```

## Behavior

### MMDB path resolution

- use `[mmdb].dir` under `XRAT_PATH` when `XRAT_PATH` is set
- otherwise default to `~/.config/xrat/mmdb`
- honor `--output` only for the current command invocation
- keep explicit custom relative `[testing.geoip]` paths resolved relative to
  `config.toml`
- keep built-in MMDB defaults resolved from `RuntimePaths.root_dir`

### Local downloader

- async `reqwest`
- edition parsing for `GeoLite2-Country`, `GeoLite2-City`, `GeoLite2-ASN`
- short aliases `country`, `city`, `asn`
- `tempfile::NamedTempFile::new_in(&mmdb_dir)` + atomic persist
- skip existing files unless `--force`
- progress bars on stderr unless `--quiet`

### Remote lookup backends

- default backend remains `mmdb`
- remote v1 backends:
  - `ipwhois`
  - `ip-api`
- backend chain built once during test-settings resolution
- cache and rate limit wrap only the remote path

## Combined Implementation Plan

### P8.1 Foundation and Read-Only CLI

Goal: establish config, path resolution, and minimal `geoip` CLI surface.

Status: in progress; mostly done.

Tasks:

- [x] Add `[mmdb]` config
- [x] Add MMDB path resolver
- [x] Wire default testing MMDB paths through shared resolver
- [x] Add `xrat geoip path`
- [x] Add `xrat geoip status`
- [x] Add parser/tests for `geoip` command family

Acceptance:

- [x] `xrat geoip path` prints the resolved MMDB directory
- [x] `XRAT_PATH=/tmp/foo xrat geoip path` resolves to `/tmp/foo/mmdb`
- [x] `xrat geoip status` lists supported editions with present/missing state

### P8.2 Downloader Core

Goal: replace the shell script for the common local MMDB workflow.

Status: in progress; core command and single-pass downloader are now implemented.

Tasks:

- [x] Add `src/app/commands/geoip/edition.rs`
- [x] Extend `src/cli/geoip.rs` with `download` and `update`
- [x] Add `src/app/commands/geoip/download.rs`
- [x] Add `src/app/commands/geoip/update.rs`
- [x] Add `AppError::GeoipDownload`
- [x] Implement single-edition async download with atomic write
- [ ] Add `--edition`, `--all`, `--output`, `--force`, `--url`, `--timeout`,
      `--quiet`
- [ ] Add focused tests for skip behavior and empty body handling
- [x] Add focused tests for parsing and URL templating

Acceptance:

- [ ] `xrat geoip download` writes `GeoLite2-Country.mmdb` under the MMDB dir
- [ ] rerun without `--force` prints a clear `skipped` message
- [ ] invalid URL override exits non-zero and includes the URL in the error

### P8.3 Multi-Edition Update

Goal: refresh all MMDB editions in one command.

Tasks:

- [ ] repeatable `--edition`
- [ ] `--all`
- [ ] concurrent downloads with bounded fan-out
- [ ] final summary `downloaded=N skipped=M failed=K`

Acceptance:

- [ ] `xrat geoip update` downloads Country, City, and ASN in one run
- [ ] one failing edition does not abort the others

### P8.4 Lookup Trait and Local Adapter

Goal: split the current sync local lookup into a reusable async abstraction.

Status: in progress; module split, trait, error type, and local adapter are now
implemented without changing call-site behavior.

Tasks:

- [x] create `src/support/geoip/` module tree
- [x] define `GeoIpLookup` trait and `GeoIpError`
- [x] move current local lookup functions into `LocalMmdbLookup`
- [x] move `classify_endpoint_location` into a separate module

Acceptance:

- [x] local MMDB lookup behavior remains unchanged
- [x] real-MMDB tests still gate on `XRAT_GEOIP_TEST_*_MMDB`

### P8.5 Remote Backends, Cache, and Rate Limit

Goal: make the lookup path pluggable for remote services.

Status: in progress; `ipwhois` is now implemented and selectable.

Tasks:

- [x] add `RemoteIpWhoisLookup`
- [ ] add `RemoteIpApiLookup`
- [ ] add cache decorator
- [ ] add rate-limit decorator
- [ ] optionally add fallback chain decorator

Acceptance:

- [ ] remote backends are selectable from `config.toml`
- [ ] repeated lookups are cached
- [ ] rate limits protect remote services during bulk runs

### P8.6 Test Integration and Inspection Commands

Goal: make the test pipeline use the resolved backend chain.

Tasks:

- [x] extend `[testing.geoip]` with `backend`, `fallback`, `remote`, `cache`
- [x] build first lookup chain during test-settings resolution
- [ ] refactor endpoint meta resolution to async trait calls
- [ ] add `xrat geoip lookup <ip>`
- [ ] add `xrat geoip backend`

Acceptance:

- [ ] `config.toml` can switch between `mmdb`, `ipwhois`, and `ip-api`
- [ ] prober keeps City -> Country -> ASN -> fallback priority

## Current Next Step

The immediate next build step is:

1. add cache and rate-limit decorators around the remote lookup path
2. add `ip-api` as the second backend
3. switch the prober to async `geoip_lookup` trait calls
4. then add `xrat geoip lookup` and `xrat geoip backend`

That is the correct next slice because the MMDB config, resolver, read-only CLI,
downloader core, and local lookup abstraction are now in place, so the remaining
work naturally splits into downloader hardening and then remote-backend
integration.

## Open Decisions

1. Keep `download` defaulting to `GeoLite2-Country`, or switch to `--all` by
   default?
2. Use `moka` + `governor`, or keep cache/rate limiting hand-rolled?
3. Ship `ip-api` in v1, or land only `ipwhois` first?
4. Add fallback chain in the same release, or after remote single-backend support
   is stable?

## Completion Criteria

Phase 8 is complete when:

1. local MMDB assets are managed under the dedicated `mmdb/` directory
2. `xrat geoip download|update|path|status` all work end-to-end
3. local MMDB remains the default lookup backend
4. remote lookup backends are selectable from `config.toml`
5. the prober uses the configured backend asynchronously
6. docs, Just recipes, and test coverage match the new behavior

### Current status against completion criteria

- Criterion 1: partially complete
  MMDB asset layout and resolver are in place; downloader core exists.
- Criterion 2: partially complete
  `download|update|path|status` exist, but downloader behavior still needs more
  hardening and integration coverage.
- Criterion 3: complete
  local MMDB remains the default backend.
- Criterion 4: partially complete
  `mmdb` and `ipwhois` are selectable in typed config; `ip-api` is still
  pending.
- Criterion 5: not complete
  resolved settings carry `geoip_lookup`, but the prober has not switched to the
  async trait yet.
- Criterion 6: not complete
  docs are tracking progress, but recipes and full test/documentation coverage
  are still pending.
