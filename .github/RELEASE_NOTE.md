## xrat v0.35.0

This release adds mouse navigation to the terminal UI and makes TUN kill switch
settings easier to change.

### Terminal UI

- Click panels, table rows, and tabs to navigate; use the mouse wheel to scroll
  the panel under the pointer. Mouse support is enabled by default and can be
  changed in Settings or with `[tui].mouse` in `config.toml`.
- Use the Settings and Keybindings controls in the bottom bar to open their
  respective dialogs. Click handling respects the actual boundaries of import,
  rename, QR, and split-tunneling dialogs.
- Show the full last-update label in the statistics header.

### TUN settings

- Cycle the kill switch through `off`, `auto`, and `strict` in the Settings
  editor. The existing runtime and firewall behavior is unchanged.

### Contributors

Thank you to [@f02xygen](https://github.com/f02xygen) for the TUN kill switch
setting in [#285](https://github.com/mhyrzt/xrat/pull/285) and mouse support in
[#286](https://github.com/mhyrzt/xrat/pull/286).

### Upgrade notes

- Mouse capture defaults to enabled. Set `mouse = false` under `[tui]` as shown
  below if you prefer terminal text selection without holding Shift.
- No new database migrations are included.
- Windows support remains experimental; automated checks do not replace
  interactive Windows and actual TUN traffic acceptance work.

```toml
[tui]
mouse = false
```

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.34.0...v0.35.0
