## xrat v0.5.1

### Performance

- **TUI**: instant first paint. GeoIP location enrichment no longer blocks the
  initial frame — known locations render immediately from the database and
  missing ones fill in progressively in the background.
- **TUI**: resolved host geo is now persisted in a database cache, so locations
  survive restarts and unresolvable or untested hosts stop re-resolving on every
  launch.
- **TUI**: mutating actions (enable/disable/delete/purge/restore) no longer
  freeze the UI. The reload runs off the event loop, and single-config toggles
  patch the affected row in place instead of reloading everything.
- **TUI**: dirty-flag rendering eliminates the constant idle redraw, and the
  engine version probe now runs asynchronously instead of delaying first paint.

### Upgrade notes

- This release adds one database migration (`0020_add_geoip_cache`) that creates
  the `geoip_cache` table. It is applied automatically on first run; no manual
  action is required.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.5.0...v0.5.1
