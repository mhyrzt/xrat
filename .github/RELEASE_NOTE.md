## xrat v0.7.0

Post-install setup now lives in the binary: a new `xrat setup` command owns
init, daemon, completions, man pages, and desktop integration, so setup works
the same regardless of how xrat was installed and can be re-run any time.

### Features

- **setup**: new `xrat setup` command runs idempotent post-install setup —
  dependency checks (`xray` required, `sing-box` optional, with versions),
  `init`, background daemon (prompted, or `--yes`), shell completions, man
  pages, and (Linux/XDG) a terminal-aware desktop launcher with icons. Flags:
  `-y/--yes`, `--no-daemon`, `--no-desktop`, `--no-completions`, `--no-manpages`,
  `--linger`.
- **setup --check**: read-only diagnostics reporting each step's status as a
  table or JSON (`--format table|json`); exits non-zero when a required step is
  missing.
- **setup output**: grouped Environment / Dependencies / Setup sections with
  status glyphs, tool versions, detected OS pretty-name, and detected desktop
  terminal.
- **linger**: interactive Linux runs prompt to enable boot-before-login start
  (systemd lingering); `--linger` forces it.
- **cargo install**: documented installing from crates.io (`cargo install xrat`
  then `xrat setup`).

### Changes

- **install.sh**: shrunk to download/verify/place the binary, then hand off to
  `xrat setup` (passing through `-y`, `--no-desktop`, `--linger`). ASCII banner
  removed.
- **release archives**: now contain just the binary, `LICENSE`, and `README`.
  Man pages, completions, and the desktop launcher/icons are generated from the
  binary via `xrat setup` (icons are embedded in the binary), so they are no
  longer bundled in the archive.

### Upgrade notes

- No new database migrations in this release.
- If you consumed `man/`, `completions/`, or `desktop/` directories from the
  release archive directly, run `xrat setup` (or `xrat manpage` /
  `xrat completions`) to generate them from the binary instead.
- Existing installs are otherwise unaffected; `xrat setup` is safe to run on an
  already-configured install and reports steps as already done.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.6.0...v0.7.0
