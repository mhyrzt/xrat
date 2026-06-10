## xrat v0.5.0

### Features

- **TUI**: faster boot — GeoIP location enrichment now runs in parallel instead
  of serially, cutting startup latency on large config sets.
- **TUI**: new page and jump scroll keys, and long log messages now wrap instead
  of being truncated.
- **Release**: release notes are now sourced from `.github/RELEASE_NOTE.md` when
  present, so the GitHub release body is controlled instead of auto-generated
  from commits.

### Fixes

- **Rotation**: `xrat rotate now` no longer selects an untested or known-bad
  config. Manual rotation now requires a passing real-delay result and otherwise
  fails with a clear message pointing at `xrat test` / `xrat scan`.
- **Rotation**: stopped persisting noisy per-candidate progress events as durable
  log rows; only bounded start/finish events and the test-run summary remain.
- **CLI**: dropped the redundant `invalid argument:` prefix from error output, so
  messages read clearly (e.g. `error: no active HTTP or SOCKS inbound; ...`).

### Upgrade notes

- No schema or config changes. Manual rotation behavior is stricter: if no
  enabled config has a passing real-delay result, `xrat rotate now` now reports
  no eligible candidate instead of rotating to an arbitrary config. Run
  `xrat test` or `xrat scan` to populate results first.
