## xrat v0.11.0

This release makes installing and launching xrat smoother.

### Features

- **cargo-binstall support.** `cargo binstall xrat` now installs the
  prebuilt release binary directly, no compiling required. `cargo install
  xrat` still works for building from source.
- **`xratui` shortcut from `xrat setup`.** The `xratui` wrapper (launches
  `xrat tui`) is now installed by `xrat setup` itself instead of only by
  `install.sh`, so `cargo install`/`cargo binstall`/manual-binary installs
  get it too after running `xrat setup`.

### Upgrade notes

- No new database migrations; safe drop-in upgrade.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.10.0...v0.11.0
