## xrat v0.9.0

REALITY proxies now work end to end.

### Fixes

- **REALITY / xhttp nodes now connect.** The runtime config generator never
  emitted a `realitySettings` block, so Xray rejected every REALITY outbound
  with `Empty "realitySettings"`. The generator now builds `realitySettings`
  (server name, public key, short id, spider-x, fingerprint) and carries the
  VLESS `flow` on the outbound user.
- **Stored configs keep their transport extensions.** Config records dropped
  `pbk`, `sid`, `fp`, `flow`, `mode`, and `alpn` when loaded from the database,
  so probes built REALITY settings with an empty public key. Xray 26.x reports
  that as `empty "password"`. Extensions are now recovered by re-parsing the
  original link, with the database columns staying authoritative.
- **Readable Xray failures.** Probe and daemon errors no longer dump the Xray
  banner, info logs, temp config path, and full module chain. The deepest
  error-chain cause is surfaced as a single line, e.g.
  `REALITY: Empty "realitySettings"`.

### Other

- The `xrat test` summary prints the `Failures:` header in red.

### Upgrade notes

- No new database migrations; safe drop-in upgrade. Previously stored REALITY
  configs start working without re-importing.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.8.1...v0.9.0
