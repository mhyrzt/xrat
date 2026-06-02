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
- [x] Added CLI parsing tests for the new `geoip` command family
- [x] Updated example config with `[mmdb]`

Not implemented yet:

- [ ] `xrat geoip download`
- [ ] `xrat geoip update`
- [ ] edition parsing (`country|city|asn` and canonical names)
- [ ] `AppError::GeoipDownload`
- [ ] atomic MMDB downloader
- [ ] remote lookup trait and module split under `src/support/geoip/`
- [ ] remote backends (`ipwhois`, `ip-api`)
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

Tasks:

- [ ] Add `src/app/commands/geoip/edition.rs`
- [ ] Extend `src/cli/geoip.rs` with `download` and `update`
- [ ] Add `src/app/commands/geoip/download.rs`
- [ ] Add `src/app/commands/geoip/update.rs`
- [ ] Add `AppError::GeoipDownload`
- [ ] Implement single-edition async download with atomic write
- [ ] Add `--edition`, `--all`, `--output`, `--force`, `--url`, `--timeout`,
      `--quiet`
- [ ] Add focused tests for parsing, URL templating, skip behavior, and empty
      body handling

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

Tasks:

- [ ] create `src/support/geoip/` module tree
- [ ] define `GeoIpLookup` trait and `GeoIpError`
- [ ] move current local lookup functions into `LocalMmdbLookup`
- [ ] move `classify_endpoint_location` into a separate module

Acceptance:

- [ ] local MMDB lookup behavior remains unchanged
- [ ] real-MMDB tests still gate on `XRAT_GEOIP_TEST_*_MMDB`

### P8.5 Remote Backends, Cache, and Rate Limit

Goal: make the lookup path pluggable for remote services.

Tasks:

- [ ] add `RemoteIpWhoisLookup`
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

- [ ] extend `[testing.geoip]` with `backend`, `fallback`, `remote`, `cache`
- [ ] build lookup chain during test-settings resolution
- [ ] refactor endpoint meta resolution to async trait calls
- [ ] add `xrat geoip lookup <ip>`
- [ ] add `xrat geoip backend`

Acceptance:

- [ ] `config.toml` can switch between `mmdb`, `ipwhois`, and `ip-api`
- [ ] prober keeps City -> Country -> ASN -> fallback priority

## Current Next Step

The immediate next build step is:

1. implement `xrat geoip download`
2. add edition parsing
3. add `xrat geoip update`

That is the correct next slice because the MMDB config, resolver, `path`, and
`status` commands are already in place and the downloader now becomes the first
missing end-user workflow.

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
