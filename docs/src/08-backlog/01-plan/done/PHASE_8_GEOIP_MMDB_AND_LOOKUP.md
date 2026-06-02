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

- The local MMDB asset-management path is established end-to-end in Rust:
  config, resolver, CLI scaffold, downloader with concurrent multi-edition
  support, update alias, lookup, and backend inspection all exist.
- The old monolithic `src/support/geoip.rs` has been split into a module tree
  with a shared trait, local MMDB adapter, backend builder, remote backends,
  decorators, and fallback chain.
- `ResolvedTestSettings` carries `geoip_lookup`, and endpoint enrichment uses
  the async trait path.
- The `geoip` CLI family is complete: `download`, `update`, `path`, `status`,
  `lookup`, and `backend` all work end-to-end.
- Large files refactored into focused module directories (`download.rs` 547→29,
  `backend.rs` 297→113, and all remaining geoip support files split).
- The old shell script removed; Justfile and README updated.
- User-facing docs created for CLI reference, config reference, and features.

Implemented:

- [x] Added top-level `[mmdb]` config in `src/app/config/mmdb.rs`
- [x] Wired `AppConfig.mmdb`
- [x] Added MMDB defaults in `src/app/config/defaults.rs`
- [x] Switched default testing GeoIP paths from `geoip/...` to `mmdb/...`
- [x] Added shared MMDB path resolution in `src/app/paths/mmdb.rs`
- [x] Made built-in MMDB defaults resolve from `RuntimePaths.root_dir`
- [x] Preserved explicit custom relative MMDB paths as config-relative
- [x] Added `xrat geoip path`
- [x] Added `xrat geoip status`
- [x] Added `xrat geoip download` with all flags (`--edition`, `--all`,
      `--output`, `--force`, `--url`, `--timeout`, `--quiet`)
- [x] Added `xrat geoip update` alias as `download --all --force`
- [x] Added `xrat geoip lookup <ip>` with `--backend`, `--no-cache`, `--json`
- [x] Added `xrat geoip backend` with `--backend`, `--no-cache`
- [x] Added MMDB edition parsing for canonical and short names
- [x] Added `AppError::GeoipDownload`
- [x] Split `src/support/geoip.rs` into `src/support/geoip/`
- [x] Added `GeoIpLookup` and `GeoIpError`
- [x] Added `LocalMmdbLookup` with preserved free-function wrappers
- [x] Moved endpoint classification into shared GeoIP support code
- [x] Extended typed `[testing.geoip]` config with backend, fallback, remote,
      and cache settings
- [x] Added the lookup-chain builder around `LocalMmdbLookup`
- [x] Added `RemoteIpWhoisLookup` and `RemoteIpApiLookup`
- [x] Wired `backend = "ipwhois"` and `backend = "ip-api"` through the builder
- [x] Added `ChainedLookup` fallback chain decorator
- [x] Added `CachedLookup` and `RateLimitedLookup` decorators
- [x] Added `geoip_lookup` to resolved test settings
- [x] Switched endpoint enrichment to use async `geoip_lookup` trait calls
- [x] Added concurrent multi-edition download with `JoinSet`
- [x] Added downloader integration tests with HTTP fixture server
- [x] Added CLI parsing tests for `geoip` command family
- [x] Updated example config with `[mmdb]`
- [x] Refactored `download.rs` (547 lines) into `download/` module
- [x] Refactored `backend.rs` (297 lines) into `backend/` module
- [x] Refactored `remote_ipwhois.rs`, `remote_ip_api.rs`, `local.rs`, `cache.rs`
      into module directories with separate tests files
- [x] Removed obsolete `scripts/download_geolite2_mmdb.sh`
- [x] Updated Justfile and README to remove shell script references
- [x] Created user-facing docs: CLI reference (`02-cli/geoip.md`), config
      reference (`[mmdb]` + `[testing.geoip]`), and feature docs
- [x] Updated sidebar and command table to include geoip

All items complete.

## Final Code Shape

Relevant files:

- `src/app/config/mmdb.rs` — `[mmdb]` config struct
- `src/app/config/testing/types.rs` — `[testing.geoip]` config types
- `src/app/paths/mmdb.rs` — MMDB path resolution helpers
- `src/cli/geoip.rs` — all geoip subcommand args
- `src/app/commands/geoip/` — command handlers
  - `path.rs`, `status.rs`, `lookup.rs`, `backend.rs`
  - `download/` — concurrent downloader module
- `src/support/geoip/` — lookup backend tree
  - `mod.rs` — `GeoIpLookup` trait, `GeoIpError`
  - `local/` — `LocalMmdbLookup`
  - `remote_ipwhois/` — `RemoteIpWhoisLookup`
  - `remote_ip_api/` — `RemoteIpApiLookup`
  - `cache/` — `CachedLookup`
  - `rate_limit.rs` — `RateLimitedLookup`
  - `chain.rs` — `ChainedLookup`
  - `backend/` — `build_lookup_chain` builder
  - `classify.rs` — endpoint location classification
- `src/app/commands/test/settings/resolve.rs` — wires `geoip_lookup`
- `src/app/commands/test/stages/endpoint.rs` — async enrichment
- `docs/src/02-cli/geoip.md` — command reference
- `docs/src/03-features/testing.md` — GeoIP enrichment docs
- `docs/src/05-reference/config-file.md` — `[mmdb]` and `[testing.geoip]`

The old shell script `scripts/download_geolite2_mmdb.sh` has been removed. The
Rust built-in downloader (via `xrat geoip download`) stores files in `mmdb/`
(not the old `geoip/` location).

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

Status: complete.

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

Status: complete.

Tasks:

- [x] Add `src/app/commands/geoip/edition.rs`
- [x] Extend `src/cli/geoip.rs` with `download` and `update`
- [x] Add `src/app/commands/geoip/download.rs` (later refactored into
      `download/`)
- [x] Add `src/app/commands/geoip/update.rs`
- [x] Add `AppError::GeoipDownload`
- [x] Implement single-edition async download with atomic write
- [x] Add `--edition`, `--all`, `--output`, `--force`, `--url`, `--timeout`,
      `--quiet`
- [x] Add focused tests for skip behavior, empty body, and HTTP fixture
- [x] Add focused tests for parsing and URL templating

Acceptance:

- [x] `xrat geoip download` writes `GeoLite2-Country.mmdb` under the MMDB dir
- [x] rerun without `--force` prints a clear `skipped` message
- [x] invalid URL override exits non-zero and includes the URL in the error

### P8.3 Multi-Edition Update

Goal: refresh all MMDB editions in one command.

Status: complete.

Tasks:

- [x] repeatable `--edition`
- [x] `--all`
- [x] concurrent downloads with bounded fan-out via `JoinSet`
- [x] final summary `downloaded=N skipped=M failed=K`

Acceptance:

- [x] `xrat geoip update` downloads Country, City, and ASN in one run
- [x] one failing edition does not abort the others

### P8.4 Lookup Trait and Local Adapter

Goal: split the current sync local lookup into a reusable async abstraction.

Status: complete.

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

Status: complete.

Tasks:

- [x] add `RemoteIpWhoisLookup`
- [x] add `RemoteIpApiLookup`
- [x] add cache decorator
- [x] add rate-limit decorator
- [x] add fallback chain decorator

Acceptance:

- [x] remote backends are selectable from `config.toml`
- [x] repeated lookups are cached
- [x] rate limits protect remote services during bulk runs
- [x] fallback chain works: primary MMDB + remote fallback

### P8.6 Test Integration and Inspection Commands

Goal: make the test pipeline use the resolved backend chain.

Status: complete.

Tasks:

- [x] extend `[testing.geoip]` with `backend`, `fallback`, `remote`, `cache`
- [x] build lookup chain during test-settings resolution
- [x] refactor endpoint meta resolution to async trait calls
- [x] add `xrat geoip lookup <ip>`
- [x] add `xrat geoip backend`

Acceptance:

- [x] `config.toml` can switch between `mmdb`, `ipwhois`, `ip-api`, and `chain`
- [x] endpoint enrichment keeps City -> Country -> ASN -> fallback priority
- [x] `xrat geoip lookup` returns country, city, and ASN for given IP
- [x] `xrat geoip backend` prints active backend configuration

### P8.7 Refactoring and Cleanup

Goal: reduce file sizes, remove obsolete code, and write user-facing docs.

Status: complete.

Tasks:

- [x] Refactor `download.rs` (547 lines) into `download/` module with `request`,
      `executor`, `progress`, `summary`, and `tests` submodules
- [x] Refactor `backend.rs` (297 lines) into `backend/` module with
      `validation.rs` and `tests.rs`
- [x] Refactor `remote_ipwhois.rs`, `remote_ip_api.rs`, `local.rs`, `cache.rs`
      into module directories with separate `tests.rs` files
- [x] Remove obsolete `scripts/download_geolite2_mmdb.sh`
- [x] Update Justfile recipes to use built-in `cargo run -- geoip download`
- [x] Remove shell-script documentation from `README.md`
- [x] Create `docs/src/02-cli/geoip.md` — full CLI command reference
- [x] Update `docs/src/05-reference/config-file.md` — add `[mmdb]` and
      `[testing.geoip]` config sections
- [x] Update `docs/src/03-features/testing.md` — replace old `geoip_mmdb` config
      with full backend documentation
- [x] Update this plan file to mark all items complete

Acceptance:

- [x] All large geoip files split into focused module directories
- [x] No remaining references to the old shell script in code or docs
- [x] 350 tests pass (55 geoip-specific, all refactored modules pass)
- [x] User-facing docs exist for CLI, config, and features

## Current Next Step

Phase 8 implementation is complete. All CLI commands, lookup backends, and
documentation are in place. The next phase can build on top of this foundation
(e.g., auto-update scheduling, additional remote providers, or broader prober
integration).

## Resolved Decisions

1. **Default editions**: `download` uses `[mmdb].default_editions` (country,
   city, asn). `--all` flag overrides to all supported editions.
2. **Cache/rate-limit implementation**: hand-rolled with `tokio::sync::Mutex`
   and `HashMap` (cache), and a simple token-bucket counter (rate limit). No
   external dependencies (`moka`, `governor`) needed.
3. **ip-api included in v1**: both `ipwhois` and `ip-api` shipped in the same
   release. Remote backends are selectable from `config.toml`.
4. **Fallback chain in same release**: `ChainedLookup` decorator shipped
   alongside single-backend support. When `backend = "chain"`, the primary is
   local MMDB with a remote fallback (`ipwhois` or `ip-api`).

## Completion Criteria

Phase 8 is complete when:

1. local MMDB assets are managed under the dedicated `mmdb/` directory
2. `xrat geoip download|update|path|status` all work end-to-end
3. local MMDB remains the default lookup backend
4. remote lookup backends are selectable from `config.toml`
5. the prober uses the configured backend asynchronously
6. docs, Just recipes, and test coverage match the new behavior

### Current status against completion criteria

- Criterion 1: complete MMDB assets managed under dedicated `mmdb/` directory.
- Criterion 2: complete `download|update|path|status` all work end-to-end with
  concurrent multi-edition support, progress bars, and HTTP fixture tests.
- Criterion 3: complete local MMDB remains the default backend.
- Criterion 4: complete `mmdb`, `ipwhois`, `ip-api`, and `chain` are all
  selectable in typed config.
- Criterion 5: complete prober uses the configured async backend for endpoint
  enrichment.
- Criterion 6: complete CLI reference, config reference, and feature docs
  created. Old shell script removed; Justfile and README updated. 350 tests pass
  across the suite (55 geoip-specific).
