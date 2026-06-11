## xrat v0.5.2

### TUI traffic dashboard

- **TUI**: the traffic tab is now a widget dashboard with a live stats tab —
  total download/upload, current throughput, delay, and failed-request counts
  fed through a ring buffer and rendered with a sparkline.
- **TUI**: engine-neutral traffic stats sources back the dashboard, plumbed for
  both engines (xray StatsService over gRPC, sing-box Clash API).

### API request logging

- **Server**: the HTTP API now records each request as a structured event.
- **TUI**: a dedicated API logs tab (moved last) splits requests into method,
  path, and status columns; stats-poll self-traffic is hidden from the engine
  and API views everywhere.

### Engine logs

- **TUI**: sing-box engine log lines are parsed into structured time/level/feed/
  source/message columns with severity colors unified across engines.
- **TUI**: the engine feed is deduped, access-log columns are filled, and the
  log card column header stays pinned while scrolling.
- **logs**: `xrat logs` output now labels the feed origin.

### View-only clears

- **TUI**: view-only log and stats clears (`C l`, `C s`) wipe the visible buffer
  without deleting persisted records, kept distinct from the DB clear (`C p`).

### Upgrade notes

- No new database migrations in this release.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.5.1...v0.5.2
