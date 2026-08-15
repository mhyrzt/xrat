## xrat v0.16.0

This release makes the existing routing settings effective in managed proxy
sessions and keeps runtime replacement safe when a candidate configuration is
invalid.

### Managed routing

- **Route directly or block traffic.** Xray and V2Ray sessions now translate
  `[routing.direct]` and `[routing.block]` domain, IP, Geosite, and GeoIP lists
  into ordered engine rules with `freedom` and `blackhole` outbounds.
- **Predictable precedence.** Direct rules run before block rules, unmatched
  traffic continues through the selected proxy, and the internal stats API
  rule remains first when statistics are enabled.
- **Route sing-box traffic safely.** sing-box sessions translate supported
  domain forms and IP/CIDR rules. Unsupported Geosite, GeoIP, and Xray-only
  forms fail with an actionable error instead of being silently ignored.
- **Keep tests isolated.** Connection-test probes and parser previews remain
  proxy-only, so routing exclusions cannot produce misleading test results.

### Runtime safety and settings

- Manual active-session replacement now runs the native Xray, V2Ray, or
  sing-box configuration validator before stopping the healthy runtime,
  matching automatic rotation safety.
- `xrat validate` rejects unsupported routing domain strategies and blank
  routing entries before launch.
- The TUI settings help and configuration documentation describe supported
  rule forms, engine-specific limitations, precedence, and PAC behavior.

### Upgrade notes

- Existing `config.toml` files remain compatible. Previously configured
  routing lists now affect managed sessions started by connect, rotation, or
  the daemon; review those lists before upgrading if they were used only for
  PAC generation.
- No database migration is required.
- Full sing-box Geosite and GeoIP rule-set translation remains planned; use
  Xray/V2Ray for those lists in this release.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.15.0...v0.16.0
