## xrat v0.16.1

This release repairs legacy database upgrades, keeps self-upgrade available
during database failures, and introduces managed routing for Xray, V2Ray, and
sing-box sessions.

### Database recovery

- Legacy v1 dedup keys are upgraded in one transaction. If multiple stored
  configs resolve to the same canonical v2 key, xrat preserves every row and
  assigns stable preservation keys instead of failing the unique constraint.
- Partially migrated databases recover automatically and repeated startup is
  idempotent. Config IDs, subscription relationships, test history, and runtime
  history are retained.
- `xrat upgrade` now runs before database initialization, so a migration or
  database connection error cannot block installing a corrective release.
- The process-introspection regression that intermittently failed release CI
  now waits for the spawned child to complete `exec` with a bounded deadline.

### Managed routing

- **Route directly or block traffic.** Xray and V2Ray sessions translate
  `[routing.direct]` and `[routing.block]` domain, IP, Geosite, and GeoIP lists
  into ordered engine rules with `freedom` and `blackhole` outbounds.
- **Predictable precedence.** Direct rules run before block rules, unmatched
  traffic continues through the selected proxy, and the internal stats API
  rule remains first when statistics are enabled.
- **Route sing-box traffic safely.** sing-box sessions translate supported
  domain forms and IP/CIDR rules. Unsupported Geosite, GeoIP, and Xray-only
  forms fail with an actionable error instead of being silently ignored.
- Connection-test probes and parser previews remain proxy-only, so routing
  exclusions cannot produce misleading test results.

### Upgrade notes

- No new SQL migration is required. The repair safely completes the v2 dedup
  backfill introduced in v0.15.0.
- If v0.15.0 reports `UNIQUE constraint failed: configs.dedup_key` before
  `xrat upgrade` can start, install this release directly:

  ```bash
  curl -fsSL https://raw.githubusercontent.com/mhyrzt/xrat/master/install.sh | bash
  ```

- Existing `config.toml` files remain compatible. Routing lists now affect
  managed sessions started by connect, rotation, or the daemon; review lists
  that were previously used only for PAC generation.
- Full sing-box Geosite and GeoIP rule-set translation remains planned; use
  Xray/V2Ray for those lists in this release.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.15.0...v0.16.1
