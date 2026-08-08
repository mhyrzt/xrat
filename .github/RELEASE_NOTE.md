## xrat v0.13.0

This release makes real-delay tests useful for region-restricted HTTP services
and tightens explicit proxy-shell protocol selection.

### Features

- **Configurable real-delay HTTP acceptance.** `[testing.real_delay]` accepts
  multiple exact status codes and inclusive ranges. Configured entries replace
  the default `200-299` policy and are combined with OR semantics.
- **Explicit redirect handling.** `follow_redirects` controls whether xrat
  checks the initial response or follows up to 10 redirects and checks the
  terminal response. Redirect loops fail with a clear diagnostic.

### Fixes

- **Strict proxy-shell protocols.** Explicit `http`, `socks5`, and `socks5h`
  selections require their matching active inbound instead of silently
  exporting another scheme. Automatic selection still cross-falls back when
  no protocol is supplied.
- **Accurate proxy-shell action status.** `enable`, `disable`, and `toggle`
  report the state produced by their emitted shell script.
- **Actionable real-delay failures.** Status mismatch messages include the test
  URL, observed status, and configured expectation.

### Upgrade notes

- No database migrations; safe drop-in upgrade.
- Existing real-delay configurations keep accepting final `200-299` responses
  and following redirects until the new fields are configured.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.12.0...v0.13.0
