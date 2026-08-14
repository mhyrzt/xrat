## xrat v0.14.0

This release brings imports and operational settings into the TUI and adds
optional subscription naming to the CLI import workflow.

### Features

- **Import from the TUI.** Press `i` from either tab to import a supported
  config share link or an HTTP(S) subscription URL. Subscription imports offer
  a compact naming step and fall back to a generated `sub-<random>` name.
- **Manage settings in the TUI.** Press `,` to browse, search, explain, edit,
  validate, and save supported `config.toml` fields without leaving xrat.
  Comment-preserving patches update only changed keys, and restart guidance is
  shown when a change cannot apply live.
- **Name CLI imports.** `xrat import` accepts optional `--name` / `-n` flags for
  new subscriptions and safe renaming when re-importing an existing URL.

### TUI improvements

- Settings use readable labels, boolean glyphs, duration units, grouped range
  values, and `Auto` for zero-valued concurrency settings.
- Contextual help documents each field's effect, accepted values, examples,
  defaults, source markers, and restart requirements.
- Responsive navigation keeps sections, values, help, save/reset behavior, and
  unsaved-change handling usable on compact terminals.

### Upgrade notes

- No database migrations; safe drop-in upgrade.
- Existing `config.toml` files remain compatible. Settings not explicitly saved
  continue using their built-in defaults.
- Database paths, binary paths, dynamic DNS hosts, GeoIP/MMDB assets, and fixed
  DNS settings remain file-managed.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.13.0...v0.14.0
