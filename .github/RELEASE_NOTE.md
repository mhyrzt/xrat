## xrat v0.5.3

### GeoIP fixes

- **geoip**: endpoint location is now derived from the server address instead of
  the proxy connection. Tested configs previously stored `loopback_ipv4` because
  the real-delay stage read the local proxy address (`127.0.0.1`); the TUI table
  Country column showed `-` while the detail card showed `loopback_ipv4`.
- **TUI**: placeholder locations are treated as missing geo, so existing
  poisoned rows self-heal from the config address off the critical path without
  affecting first-paint time.

### Upgrade notes

- No new database migrations in this release.
- Re-run `xrat test` (or let the TUI re-enrich in the background) to replace any
  `loopback_ipv4` locations stored by earlier versions with real geo.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.5.2...v0.5.3
