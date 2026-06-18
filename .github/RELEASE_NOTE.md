## xrat v0.8.1

Small follow-up to v0.8.0.

### Fixes

- **TUI help modal**: combine the tab (`[ / ]`) and card-focus (`⇥ / ⇤`) rows
  onto single lines and pad the key column by display width, so multi-glyph keys
  line up with a consistent gap.

### Internal

- **CI**: bump `actions/upload-artifact` and `actions/download-artifact` to v5
  (Node 24), clearing the remaining Node 20 deprecation warnings in the release
  workflow.

### Upgrade notes

- No new database migrations and no behavior changes; safe drop-in upgrade.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.8.0...v0.8.1
