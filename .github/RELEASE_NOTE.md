## xrat v0.8.0

Follow-up to v0.7.0: fixes the crates.io publish and reworks the TUI hotkeys for
API sharing and source refresh.

### Fixes

- **crates.io**: `xrat setup` embedded its desktop `.desktop` template and icons
  from `packaging/`/`docs/`, which are excluded from the published crate, so the
  v0.7.0 `cargo publish` verify build failed. The assets are now vendored under
  `src/` and ship in the crate. (v0.7.0 was tagged with a GitHub release but
  never reached crates.io; install via `cargo install xrat` works from v0.8.0.)

### Features

- **TUI hotkeys**: API sharing moves to an `a` chord available in all main views
  — `a q` shows the API QR, `a c` copies the API link — replacing the
  Sources-only `u`/`U` bindings. `u` is now a global "update all sources" key,
  and the Sources-only `R` refresh-all binding is removed. The help modal
  reflects the new bindings.

### Internal

- **CI**: bump `actions/checkout` to v5 (Node 24) to clear the Node 20
  deprecation warning.

### Upgrade notes

- No new database migrations in this release.
- TUI users: the old Sources `u`/`U`/`R` keys are gone — use `a q` / `a c` for
  API sharing (any view) and `u` to update all sources.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.7.0...v0.8.0
