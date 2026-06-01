# Phase 8.1: Remote GeoIP Lookup Backends

## Goal

Make the GeoIP lookup path pluggable so users can choose between the existing
local MaxMind MMDB files and a remote GeoIP service such as `ipwhois.io`,
without changing the rest of the prober pipeline. By the end of this phase XRAT
should be able to:

- pick the lookup backend through `config.toml` (`mmdb`, `ipwhois`, `ip-api`,
  ...) without code changes
- keep the current MMDB path as the default so existing users see no change
- transparently cache remote lookups in memory with a configurable TTL
- rate-limit remote backends so the free tier of a public service is not
  exhausted by a single bulk test run
- fall back from one backend to another (e.g. MMDB first, remote second) through
  a chain that the operator declares
- expose the active backend and a one-off `lookup` command for inspection
- keep the real-MMDB test surface working unchanged

## Why This Phase Exists

The current GeoIP enrichment in `src/support/geoip.rs` only knows about local
MaxMind MMDB files. That is fine for users who already have a MaxMind license
key and can download the official databases, but it leaves out the much larger
group of users who just want enrichment without maintaining a 70+ MiB MMDB
directory. Public services like `ipwhois.io` and `ip-api.com` already expose
country / region / city / ASN / org data and are good enough for the "where is
this proxy endpoint?" question that the `xrat test` stage already asks.

The downloader work in Phase 8 covers the "I want a local MMDB" path. This phase
covers the "I do not want a local MMDB" path so the two are first-class options
in the same CLI.

The prober today calls `geoip::lookup_*` synchronously and reads three separate
files. Making the lookup async and trait-based does not change the public
behavior; it just lets the implementation point at something that is not on the
local filesystem.

## Current Starting Point

Relevant pieces already in the codebase:

- `src/support/geoip.rs` exposes sync `lookup_country_iso`, `lookup_city_label`,
  `lookup_asn_label` over `maxminddb::Reader`
- `src/app/commands/test/stages/endpoint.rs::resolve_endpoint_meta` is the only
  consumer today; it tries City first, then Country, then ASN, and falls back to
  a `classify_endpoint_location` heuristic
- `[testing.geoip]` in `src/app/config/testing/types.rs` carries `enabled`,
  `country_path`, `city_path`, `asn_path`
- `src/app/config/testing/default_values.rs::GeoIpTestSettings::default` pins
  the canonical paths to `geoip/GeoLite2-{Country,City,ASN}.mmdb`
- `src/app/commands/test/settings/resolve.rs` resolves those paths against the
  config directory and stuffs them into `ResolvedTestSettings`
- real-MMDB tests in `src/support/geoip.rs` and
  `src/app/commands/test/tests/geoip_cases/` are gated behind
  `XRAT_GEOIP_TEST_{,CITY,ASN}_MMDB` and must keep working unchanged
- `reqwest` is already a dependency with `rustls` and async support
- `Cargo.toml` is on `edition = "2024"`, so native `async fn` in traits is
  available
- `indicatif` and `tempfile` are already present (useful for CLI progress and
  for any persistent cache work)

## Scope Boundary

Phase 8.1 should cover:

- a `GeoIpLookup` async trait in `src/support/geoip/` with three aspects
  (`country`, `city`, `asn`) and a `backend_name` accessor
- a `LocalMmdbLookup` adapter that wraps the current sync functions (identical
  behavior, just async-shaped)
- a `RemoteIpWhoisLookup` implementation targeting `https://ipwhois.io`
- a `RemoteIpApiLookup` implementation targeting `http://ip-api.com` as a second
  example so the trait is provably multi-implementation
- a `CachedLookup` decorator with TTL and size cap
- a `RateLimitedLookup` decorator with a per-minute budget
- an optional `ChainedLookup` decorator that tries a primary and falls back to a
  secondary on `Ok(None)` or transient error
- extended `[testing.geoip]` config schema with `backend`,
  `[testing.geoip.remote]`, and `[testing.geoip.cache]` sections
- selection logic in `resolve_test_settings` that builds the right backend chain
  at startup
- prober integration in `stages/endpoint.rs` that awaits the trait instead of
  calling the sync functions
- two new `xrat geoip` subcommands for inspection: `lookup <ip>` and `backend`
- focused tests for the trait, each backend, the decorators, and the config
  schema

Phase 8.1 should not yet cover:

- a persistent on-disk cache (in-memory is enough for a single test run and
  avoids a migration story for the cache file)
- an auto-update scheduler for remote backends (not applicable; they are
  pull-on-demand)
- MaxMind licensed direct downloads (separate auth/key story; out of scope)
- the TUI "test enrichment" live view (can call the same service later)
- an HTTP API endpoint for one-off lookups (no obvious consumer)
- IPv6-only quirks that some remote services have (handle the common case and
  document the limitations)

## Service Catalog for v1

These are the services this phase plans to ship as first-class backends. Each
one is a small adapter that implements `GeoIpLookup`.

### `ipwhois` (primary)

- Endpoint: `GET https://ipwhois.app/json/{ip}` (HTTPS, no key required for the
  free tier)
- Returns: `country`, `country_code`, `region`, `city`, `connection.asn`,
  `connection.org`, `connection.isp`
- Maps to:
  - `country` -> `country_code`
  - `city` -> `City/Country` (region omitted for label parsimony)
  - `asn` -> `AS{n} {org}` when `connection.asn` is present
- Auth: none for the public free endpoint. Plan should reserve an `api_key`
  config field for the future Pro tier without surfacing it yet
- Rate limit guidance: stay well below the documented free tier (a conservative
  30 req/min default is safer than the documented limit; the cache makes the
  effective rate much lower)
- Reference: <https://ipwhois.app>

### `ip-api` (secondary)

- Endpoint: `GET http://ip-api.com/json/{ip}` (HTTP, no key for the free
  non-commercial tier)
- Returns: `country`, `countryCode`, `region`, `regionName`, `city`, `isp`,
  `org`, `as`
- Maps to:
  - `country` -> `countryCode`
  - `city` -> `City/Region/Country`
  - `asn` -> `AS{as} {org}` when `as` parses
- The HTTP endpoint is documented as the standard for non-commercial personal
  use. A `https` URL override is supported in case the service changes its
  policy
- Rate limit guidance: 45 req/min by default per the public guidance
- Reference: <https://ip-api.com>

### Future services (out of scope for v1)

`ipinfo.io`, `ipapi.is`, `ipapi.co`, and `ip-api.io` are noted as future
adapters. The trait design must make adding them a small, self-contained file
with no shared parsing logic.

## CLI Entry

The new subcommands extend the `xrat geoip` family from Phase 8. They reuse the
same `geoip` parent so the command surface stays compact.

### `xrat geoip lookup <ip>`

Look up a single IP through the configured backend chain and print the result.
Useful for sanity-checking the backend without running a full test batch.

| Flag               | Description                                                                                     |
| ------------------ | ----------------------------------------------------------------------------------------------- |
| `--backend <name>` | Override `backend` for this invocation. Accepts `mmdb`, `ipwhois`, `ip-api`, or `chain:<a>,<b>` |
| `--no-cache`       | Bypass the in-memory cache. Useful when verifying that a remote service is actually responsive. |
| `--json`           | Emit machine-readable JSON instead of the default text table.                                   |

Default text output (one row per aspect that returned data):

```text
backend: ipwhois
country: NL
city:    Amsterdam/NL
asn:     AS60781 LeaseWeb
```

Exits with non-zero status when the configured backend cannot resolve the IP
(network error, rate limit, parse error).

### `xrat geoip backend`

Print the active backend chain as resolved from `config.toml` and the process
environment. Example:

```text
primary:    mmdb
fallback:   ipwhois
cache:      enabled (ttl=24h, max=10000)
rate limit: 30 req/min (only applies to ipwhois)
```

Exits zero. Requires the same config as `xrat test` (so it is useful for
debugging "why is my enrichment empty?").

## Module Layout

Recommended initial layout under `src/support/geoip/`:

```text
src/support/geoip/
  mod.rs                 # re-exports + GeoIpLookup trait + GeoIpError
  local.rs               # LocalMmdbLookup wrapping the existing sync functions
  remote_ipwhois.rs      # RemoteIpWhoisLookup + wiremock-friendly constructor
  remote_ip_api.rs       # RemoteIpApiLookup
  cache.rs               # CachedLookup decorator
  rate_limit.rs          # RateLimitedLookup decorator
  chain.rs               # ChainedLookup decorator
  backend.rs             # build_lookup_chain(settings) -> Arc<dyn GeoIpLookup>
  classify.rs            # the existing classify_endpoint_location helper
```

The existing `src/support/geoip.rs` is split:

- `lookup_country_iso` / `lookup_city_label` / `lookup_asn_label` move to
  `local.rs` as methods on `LocalMmdbLookup`
- `classify_endpoint_location` moves to `classify.rs`
- the real-MMDB tests stay reachable from `local.rs` tests module

`src/app/commands/geoip/` (from Phase 8) gets two new files:

```text
src/app/commands/geoip/
  lookup.rs              # xrat geoip lookup
  backend.rs             # xrat geoip backend
```

Config additions live next to the existing `testing` settings:

```text
src/app/config/testing/
  types.rs               # extended GeoIpTestSettings + new sub-structs
  default_values.rs      # defaults for the new fields
```

## Trait Design

The trait should be small, async, and `Send + Sync` so it can sit behind
`Arc<dyn GeoIpLookup>` in the resolved test settings.

```rust
#[async_trait::async_trait]
pub trait GeoIpLookup: Send + Sync {
    async fn country(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError>;
    async fn city(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError>;
    async fn asn(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError>;
    fn backend_name(&self) -> &'static str;
}
```

Notes:

- Using `#[async_trait]` keeps `dyn GeoIpLookup` compatible. Native `async fn`
  in traits would require the `trait-variant` workaround; the small macro cost
  is worth the simpler object-safe surface
- The trait returns `Result<Option<...>>` so transient errors (rate limit, HTTP
  5xx, parse failure) are distinguishable from "this backend does not know about
  this IP" which returns `Ok(None)`
- `GeoIpError` is a small enum (see Error Handling) and converts to `AppError`
- Each aspect is queried independently so a backend that only has country data
  does not have to lie about city/asn

### LocalMmdbLookup

Wraps the three existing sync functions. Each method opens the
`maxminddb::Reader` on demand, parses, and converts to a `City` /
`Country`-shaped label. The behavior is byte-for-byte identical to today.

Rationale for keeping the local backend as a struct instead of inlining it: it
gives the test path a uniform async surface, and it lets the cache decorator
wrap the local backend too (cheap wins when many configs share an endpoint IP,
e.g. CDN-hosted proxies).

### RemoteIpWhoisLookup

- holds a `reqwest::Client` (cloned per lookup)
- holds the configured endpoint, timeout, and optional api_key
- one `GET /json/{ip}` per aspect is wasteful; fetch once and project three
  fields from the response
- response schema (excerpt):

  ```json
  {
    "ip": "1.2.3.4",
    "country": "Netherlands",
    "country_code": "NL",
    "region": "North Holland",
    "city": "Amsterdam",
    "connection": {
      "asn": 60781,
      "org": "LeaseWeb"
    }
  }
  ```

- map:
  - `country` -> `country_code`
  - `city` -> `"City/Country"` when both present, else `None`
  - `asn` -> `format!("AS{} {}", asn, org)` when `asn` is set
- network errors -> `GeoIpError::Http`
- non-2xx -> `GeoIpError::Status { status, body_preview }`
- malformed body (missing fields, wrong types) -> `GeoIpError::Parse`
- the cache layer is responsible for collapsing duplicate IPs into a single
  request; this layer just does one HTTP call per aspect invocation (callers are
  expected to call it from inside `CachedLookup::country`/`city`/`asn`)

### RemoteIpApiLookup

Same shape as `RemoteIpWhoisLookup`, with the response schema documented in the
`service_catalog` section. The two adapters share no parsing code so a
service-specific bug stays in one file.

### CachedLookup

- wraps `Arc<dyn GeoIpLookup>`
- keyed by `(aspect, ip)` string
- stores `String -> (value: Option<String>, expires_at: Instant)`
- bounded by `max_entries` (default 10_000); on overflow, evict the earliest
  `expires_at` (FIFO is fine for v1; LRU is a later polish)
- TTL defaults to 24h; configurable via `[testing.geoip.cache].ttl_secs`
- `cache_hit` increments a counter that the Diagnostics view can show later; not
  user-facing in v1

Recommended crate: `moka` with the `future` feature. It handles size-bound
eviction and TTL expiry without a manual cleanup task. If adding a crate is
undesirable, a `Mutex<HashMap<...>>` with a periodic `tokio::spawn` cleanup is
acceptable but more code to test.

### RateLimitedLookup

- wraps `Arc<dyn GeoIpLookup>`
- uses `governor::RateLimiter` keyed on the backend name (not per IP; the
  limiter protects the service, not the cache)
- per-minute budget defaults to 30 (ipwhois) and 45 (ip-api) and is configurable
  under `[testing.geoip.remote].rate_limit_per_minute`
- on rejection: `GeoIpError::RateLimited { retry_after }`
- cache hits bypass the limiter (the limiter is a wrapper _inside_ the cache
  wrapper, so a cached response never reaches the limiter)

### ChainedLookup (optional, design-only for v1)

- wraps a `primary: Arc<dyn GeoIpLookup>` and a `fallback: Arc<dyn ...>`
- on `Ok(None)` from the primary, returns the fallback result
- on `Err(GeoIpError::Parse | GeoIpError::Status { 4xx })` from the primary,
  returns the fallback result
- on `Err(GeoIpError::Http | GeoIpError::RateLimited)` from the primary, returns
  the primary error (transient errors should not silently downgrade to a remote
  call)
- this decorator is the most useful for the "MMDB first, remote as backup"
  workflow

## Config Schema

The existing `[testing.geoip]` block grows but stays backward compatible.

```toml
[testing.geoip]
enabled = true
# Backend selector. One of:
#   "mmdb"     - local MaxMind files (default; matches pre-8.1 behavior)
#   "ipwhois"  - ipwhois.app/json/{ip}
#   "ip-api"   - ip-api.com/json/{ip}
#   "chain"    - use [testing.geoip.fallback] when the primary returns None
backend = "mmdb"

# MMDB paths. Used when backend = "mmdb" or as the primary in a chain.
country_path = "geoip/GeoLite2-Country.mmdb"
city_path = "geoip/GeoLite2-City.mmdb"
asn_path = "geoip/GeoLite2-ASN.mmdb"

# Optional fallback backend. Only consulted when backend = "chain".
# Same values as `backend`, or "none" to disable.
fallback = "none"

# Remote backend settings. Used when backend is a remote service or as
# the fallback in a chain.
[testing.geoip.remote]
provider = "ipwhois"      # one of "ipwhois", "ip-api"
endpoint = ""             # override base URL (defaults per provider)
timeout_ms = 5000
api_key = ""              # reserved for future Pro/paid tiers
rate_limit_per_minute = 30

# In-memory cache for remote lookups.
[testing.geoip.cache]
enabled = true
ttl_secs = 86400          # 1 day
max_entries = 10000
```

Notes:

- `#[serde(default)]` on the new sub-tables means old configs continue to parse
  and pick up the new defaults
- `backend = "mmdb"` is the default, so the behavior of an unmodified config is
  identical to before
- The schema explicitly reserves `api_key` and a `secrets` story is
  intentionally deferred (see Open Questions)
- An invalid `backend` value fails startup with a clear
  `AppError::InvalidArgument` listing the valid values

### `fallback` semantics

- `fallback = "none"` (default): single backend, no chain
- `fallback = "mmdb"`: use the primary remote backend; if it returns `Ok(None)`
  for an aspect, fall back to the local MMDB
- `fallback = "ipwhois"`: the primary is MMDB, the secondary is `ipwhois`
- `fallback` cannot equal `backend`; the resolver rejects `chain:chain`
  configurations at startup

## Settings Resolve

`src/app/commands/test/settings/resolve.rs` is the place where the backend chain
is constructed. The current shape returns `geoip_country_path`,
`geoip_city_path`, `geoip_asn_path` on `ResolvedTestSettings`. After Phase 8.1:

- keep the three path fields so existing call sites that read them (e.g.
  real-MMDB tests) keep working
- add `geoip_lookup: Arc<dyn GeoIpLookup>` carrying the constructed chain
- the construction calls a
  `build_lookup_chain(&GeoIpTestSettings, &RuntimePaths) -> Result<Arc<dyn GeoIpLookup>, AppError>`
  helper from `src/support/geoip/backend.rs`
- the chain is built once per `AppContext`, not per test row

The prober in `stages/endpoint.rs` switches from the sync functions to the trait
method calls. The City -> Country -> ASN -> fallback order is preserved by
querying the trait in that order.

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum GeoIpError {
    #[error("geoip HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("geoip service returned status {status} for {ip}")]
    Status { ip: String, status: u16, body_preview: String },
    #[error("geoip service response was malformed: {0}")]
    Parse(String),
    #[error("geoip rate limit exceeded; retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("geoip backend not configured")]
    NotConfigured,
}
```

`From<GeoIpError> for AppError` is added so callers can use `?` without extra
ceremony. `RateLimited` is surfaced to the user as a non-fatal warning in the
test run output (the row still records what it has and the run continues); all
other variants are fatal for that aspect (the prober falls back to the next
aspect or to `classify_endpoint_location`).

## Implementation Slices

### P8.1.1 Trait, Error, and LocalMmdbLookup

Goal: introduce the trait and the local adapter without changing behavior.

Tasks:

- [ ] Create `src/support/geoip/` module tree
- [ ] Define `GeoIpLookup` trait, `GeoIpError`, and `From` into `AppError`
- [ ] Move the existing `lookup_*` functions into `local.rs::LocalMmdbLookup`
- [ ] Move `classify_endpoint_location` into `classify.rs`
- [ ] Update `support/mod.rs` to re-export the new module structure
- [ ] Add unit tests for `LocalMmdbLookup` that mirror the existing real-MMDB
      tests so coverage does not regress
- [ ] Confirm `cargo test -q geoip::` still passes against the existing
      `XRAT_GEOIP_TEST_*_MMDB` env vars

Acceptance:

- [ ] existing prober integration still works (no behavior change yet)
- [ ] the real-MMDB tests still gate behind the same env vars
- [ ] `LocalMmdbLookup::backend_name()` returns `"mmdb"`

### P8.1.2 RemoteIpWhoisLookup

Goal: a working remote adapter for the user-mentioned service.

Tasks:

- [ ] Add `remote_ipwhois.rs` with `RemoteIpWhoisLookup`
- [ ] Map the JSON response to the three trait methods
- [ ] Add `wiremock` (dev dependency) and write a test that - returns a stubbed
      `ipwhois.app/json/8.8.8.8` response - asserts the three aspects resolve
      correctly - asserts a 4xx produces `GeoIpError::Status` - asserts a
      malformed body produces `GeoIpError::Parse`
- [ ] Add a live smoke test gated behind `XRAT_GEOIP_REMOTE_LIVE=1` so the live
      service is never hit during regular `cargo test`

Acceptance:

- [ ] `RemoteIpWhoisLookup` round-trips against the wiremock fixture
- [ ] one real-network smoke test confirms the live service is reachable for at
      least one well-known IP

### P8.1.3 RemoteIpApiLookup

Goal: prove the trait supports multiple backends with a second implementation.

Tasks:

- [ ] Add `remote_ip_api.rs` mirroring the structure of `remote_ipwhois.rs`
- [ ] Wiremock test for the documented JSON shape
- [ ] `backend_name()` returns `"ip-api"`

Acceptance:

- [ ] both adapters compile behind the same trait without sharing parsing
      helpers
- [ ] wiremock test passes for `ip-api`

### P8.1.4 CachedLookup Decorator

Goal: collapse repeated lookups in a single test run.

Tasks:

- [ ] Add `cache.rs` with `CachedLookup` and configurable TTL + `max_entries`
- [ ] Unit tests: - first call hits the inner backend - second call within TTL
      returns the cached value - third call after TTL eviction hits the inner
      backend again - `max_entries` cap is enforced
- [ ] Decide between `moka` and a hand-rolled `HashMap` and document the
      decision in the slice PR

Acceptance:

- [ ] cache tests pass
- [ ] `CachedLookup` works around both `LocalMmdbLookup` and
      `RemoteIpWhoisLookup` in a small integration test

### P8.1.5 RateLimitedLookup Decorator

Goal: protect remote backends from bulk test bursts.

Tasks:

- [ ] Add `rate_limit.rs` with `RateLimitedLookup` and a per-minute budget
- [ ] Unit tests: - under-budget calls pass through - over-budget calls return
      `GeoIpError::RateLimited` - cache hits bypass the limiter (verified by
      counting `governor` outcomes or by inspecting the limiter's view)
- [ ] Default budget per provider: - `ipwhois`: 30/min - `ip-api`: 45/min
- [ ] Use `governor` or a hand-rolled token bucket; document the decision in the
      slice PR

Acceptance:

- [ ] rate-limit tests pass
- [ ] chained `(Cached -> RateLimited -> Remote)` ordering is correct (cache hit
      does not consume the rate budget)

### P8.1.6 Config Schema Extension

Goal: wire the new fields into the typed config and defaults.

Tasks:

- [ ] Extend `GeoIpTestSettings` with `backend`, `fallback`,
      `remote: RemoteGeoIpSettings`, `cache: GeoIpCacheSettings`
- [ ] Add `RemoteGeoIpSettings` and `GeoIpCacheSettings` structs
- [ ] Add `Default` impls in `default_values.rs`
- [ ] Add `#[serde(default)]` on the new sub-tables
- [ ] Add validation in a new `validate_geoip_settings` function: - `backend`
      and `fallback` are in the allowed set - `fallback != backend` when both
      are set - `cache.ttl_secs` and `cache.max_entries` are positive when
      `cache.enabled` is true
- [ ] Add config-parsing tests for the new fields, including a round-trip that
      confirms old configs still parse

Acceptance:

- [ ] an existing `config.toml` without the new fields parses identically
- [ ] an invalid `backend = "nope"` fails at startup with a clear error
- [ ] `fallback = "ipwhois"` with `backend = "ipwhois"` is rejected

### P8.1.7 Backend Builder and Resolved Settings

Goal: one place constructs the chain from typed config.

Tasks:

- [ ] Add `src/support/geoip/backend.rs::build_lookup_chain`
- [ ] Add `geoip_lookup: Arc<dyn GeoIpLookup>` to `ResolvedTestSettings`
- [ ] Keep the three path fields so downstream readers do not break
- [ ] Add a test that asserts the right chain shape is built for each
      combination: - `backend = "mmdb"` -> `LocalMmdbLookup` -
      `backend = "ipwhois"` -> `Cached(RateLimited(RemoteIpWhois))` -
      `backend = "chain", fallback = "ipwhois"` ->
      `Chain(Local,       Cached(RateLimited(RemoteIpWhois)))`
- [ ] Wire the builder call in `resolve_test_settings` so the prober can `await`
      the trait

Acceptance:

- [ ] each backend combination is unit-tested
- [ ] no other test code needs to know about the construction

### P8.1.8 Prober Integration

Goal: the prober goes async and uses the trait.

Tasks:

- [ ] Refactor `resolve_endpoint_meta` in
      `src/app/commands/test/stages/endpoint.rs` to `async fn` and take
      `&dyn GeoIpLookup` instead of three paths
- [ ] Preserve the City -> Country -> ASN -> fallback priority
- [ ] Preserve the existing return shape
      (`EndpointMeta { location,     country, asn }`)
- [ ] Update the bulk executor to `await` the now-async function
- [ ] Add a focused test that stubs the trait and asserts the priority order
      with a fixture backend

Acceptance:

- [ ] existing real-MMDB tests pass without modification
- [ ] a new fixture test confirms the priority order
- [ ] the prober awaits a single trait call per aspect per IP

### P8.1.9 CLI Inspection Commands

Goal: give users a way to see what is configured and a way to query one IP
without running a full test.

Tasks:

- [ ] Add `src/app/commands/geoip/lookup.rs::run` with `<ip>` argument and
      `--backend`, `--no-cache`, `--json` flags
- [ ] Add `src/app/commands/geoip/backend.rs::run` printing the resolved chain
- [ ] Extend `src/cli/geoip.rs` with the two new variants
- [ ] Add CLI parsing tests for both commands
- [ ] Add an integration test that runs `lookup` against a wiremock-backed
      context

Acceptance:

- [ ] `xrat geoip lookup 8.8.8.8 --backend ipwhois --no-cache` hits wiremock and
      prints the result
- [ ] `xrat geoip backend` prints the active chain and exits

### P8.1.10 Docs and Recipes

Goal: teach the new option.

Tasks:

- [ ] Extend `docs/src/02-cli/geoip.md` with `lookup` and `backend` sections
- [ ] Extend `docs/src/03-features/testing.md` GeoIP section with a "Remote
      backends" subsection
- [ ] Document `[testing.geoip.remote]` and `[testing.geoip.cache]` in
      `docs/src/05-reference/config-file.md`
- [ ] Add a `just geoip-lookup` recipe for a quick smoke
- [ ] Add a `just geoip-backend` recipe that prints the active chain
- [ ] Update `README.md` GeoIP section with a short example of switching to a
      remote backend

Acceptance:

- [ ] mdbook builds
- [ ] recipes work end-to-end

## Documentation

Update when the phase starts:

- `docs/src/SUMMARY.md` if a new sub-page is added
- `docs/src/02-cli/geoip.md` (Phase 8 page) for `lookup` and `backend`
- `docs/src/03-features/testing.md` for the remote-backend story
- `docs/src/05-reference/config-file.md` for the schema additions
- `README.md` GeoIP section for a quick example

## Open Questions

1. **Service catalog for v1** — ship `ipwhois` and `ip-api`, or just `ipwhois`
   and add others as follow-ups? This plan assumes both so the trait is provably
   multi-implementation. If a smaller surface is preferred, drop `ip-api` and
   the `P8.1.3` slice.

2. **Async trait approach** — `#[async_trait]` macro (chosen here for
   `dyn`-safety) versus native `async fn` in traits with a `trait-variant`
   workaround, versus a sealed enum dispatch. The macro is the lowest-friction
   option but adds a small dep. Worth confirming before slice 8.1.1 lands.

3. **Crate picks for cache and rate limit** — `moka` + `governor` (chosen here
   for ergonomics) versus hand-rolled `HashMap` and token bucket. Hand-rolled
   avoids two new deps but adds code to test. Worth confirming before slice
   8.1.4 lands.

4. **Fallback chain in v1** — ship `ChainedLookup` in the same release, or defer
   to a later slice? It is the most ergonomic way to say "MMDB first, remote as
   backup", so this plan includes it. If scope is too large, drop the `chain`
   validator field and the `chain.rs` file.

5. **Persistent cache** — in-memory only (chosen here) or on-disk cache too? An
   on-disk cache would survive restarts and is friendlier for the daemon
   auto-update path that Phase 3p5 mentions. Out of scope for v1 to avoid a
   migration story for the cache file; revisit if user feedback asks for it.

6. **Rate limit defaults** — the conservative defaults (30/min for `ipwhois`,
   45/min for `ip-api`) are below the documented free tiers. Confirm with the
   service terms at implementation time rather than hard-coding assumptions in
   the plan.

7. **API key handling** — `api_key` is reserved in the schema but not used yet.
   When the time comes, decide between env var (`XRAT_GEOIP_API_KEY`),
   `config.toml` plaintext, or a `secrets` story. Defer to a later phase.

8. **TUI integration** — should the existing TUI Phase 6 work surface the
   resolved backend in the Diagnostics view? It can call the same
   `build_lookup_chain` once `xrat geoip backend` exists. Low-cost follow-up,
   not blocking 8.1.

9. **Per-IP behavior on rate limit** — when `GeoIpError::RateLimited` fires, do
   we want to (a) keep going and use the fallback chain, (b) retry with backoff
   for the same IP, or (c) surface a warning and continue without enrichment?
   This plan picks (c) for simplicity; revisit if the warning turns out to be
   too noisy in bulk runs.

10. **Real-MMDB test path** — the existing tests gate behind
    `XRAT_GEOIP_TEST_*_MMDB`. Phase 8.1 should not change those env vars.
    Confirm in P8.1.1 that the gating still works after the module split.

## Completion Criteria

Phase 8.1 can be considered complete when:

1. `config.toml` can switch the GeoIP backend from `mmdb` to `ipwhois` (or
   `ip-api`) with no other code changes.
2. An unmodified `config.toml` (no `backend` field) still produces the same
   enrichment as before.
3. The prober queries the configured backend asynchronously and preserves the
   City -> Country -> ASN -> fallback priority.
4. The in-memory cache collapses repeated IPs within a TTL.
5. The rate limiter prevents more than the configured per-minute budget of
   remote calls.
6. `xrat geoip lookup <ip>` works against any configured backend, including
   `--no-cache`.
7. `xrat geoip backend` prints the resolved chain.
8. Real-MMDB tests (`XRAT_GEOIP_TEST_*_MMDB`) still pass.
9. `cargo fmt` and `cargo test -q` pass, including the new `geoip::` tests.
10. mdbook builds and the new config options are documented.
11. CI does not hit the live services; live smoke is gated behind an env var.

## Out of Scope

- MaxMind direct downloads (license key, account, separate auth flow)
- A persistent on-disk cache
- Auto-update scheduling for remote backends (not applicable; they are
  pull-on-demand)
- An HTTP API endpoint for one-off lookups
- A TUI panel/button for the new backends
- IPv6-specific quirks beyond what the services already handle
- `api_key` plumbing for the future Pro/paid tiers
- New services beyond `ipwhois` and `ip-api` in v1
