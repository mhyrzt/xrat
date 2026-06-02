<p align="center">
  <img src="media/xrat-hero.png" alt="xrat" width="600">
</p>

<h1 align="center">XRAT</h1>

<p align="center">
  <em>A modern proxy configuration manager for Xray-core and sing-box</em>
</p>

<p align="center">
  <img alt="Status" src="https://img.shields.io/badge/status-under%20development-orange">
  <img alt="Rust" src="https://img.shields.io/badge/rust-stable-blue">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green">
</p>

XRAT is a Rust CLI for importing, storing, testing, and running Xray-compatible
proxy configurations.

## Current Features

- Import subscription/config lines into SQLite or PostgreSQL.
- List stored configs and subscriptions.
- Test configs for connectivity, latency, and optional download speed.
- Run one stored config as a managed local Xray runtime.
- Show runtime status and stop the active runtime session.
- Parse and validate links without importing, with optional JSON output.

## Development

Common commands:

```bash
cargo build
cargo test -q
cargo fmt
```

Run the CLI locally:

```bash
cargo run -- import <input>
cargo run -- list configs
cargo run -- test
cargo run -- connect <id>
cargo run -- status
cargo run -- disconnect
cargo run -- parse 'vless://...'
cargo run -- parse --json --engine auto 'vless://...'
cargo run -- parse --json --engine auto 'hy2://...'
```

## Documentation

Planning notes and validation checklists live in `docs/src/backlog/`.

## GeoIP Database (Optional)

XRAT stores app state under `~/.config/xrat` by default (or `XRAT_PATH` when
set). For GeoIP databases, use a dedicated subfolder:

- `~/.config/xrat/geoip/GeoLite2-Country.mmdb`
- `~/.config/xrat/geoip/GeoLite2-City.mmdb`

GeoLite2 files can be downloaded from:

- <https://github.com/P3TERX/GeoLite.mmdb/>

Use the built-in download command:

```bash
xrat geoip download
```

Optional test-time GeoIP lookup config:

```toml
[testing.geoip]
enabled = true
country_path = "geoip/GeoLite2-Country.mmdb"
city_path = "geoip/GeoLite2-City.mmdb"
asn_path = "geoip/GeoLite2-ASN.mmdb"
```

GeoIP enrichment order: City -> Country -> ASN -> fallback classifier. Paths can
be relative to your config file location.

Optional real-MMDB test:

```bash
XRAT_GEOIP_TEST_MMDB=./testdata/xrat/geoip/GeoLite2-Country.mmdb \
  cargo test -q looks_up_country_from_real_mmdb_when_provided
```

City/ASN real-MMDB tests:

```bash
XRAT_GEOIP_TEST_CITY_MMDB=./testdata/xrat/geoip/GeoLite2-City.mmdb \
  cargo test -q looks_up_city_from_real_mmdb_when_provided

XRAT_GEOIP_TEST_ASN_MMDB=./testdata/xrat/geoip/GeoLite2-ASN.mmdb \
  cargo test -q looks_up_asn_from_real_mmdb_when_provided
```
