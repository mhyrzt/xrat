## xrat v0.15.0

This release preserves imported proxy parameters end to end and makes automatic
rotation safer when an active connection becomes unhealthy.

### Config compatibility

- **Round-trip imported parameters.** VLESS, VMess, Trojan, Shadowsocks, HTTP,
  SOCKS5, and Hysteria2 imports retain non-structural parameters, native VMess
  JSON values, and repeated URL query keys instead of silently dropping them.
- **Generate current Xray transports.** Runtime generation covers current raw,
  WebSocket, gRPC, xhttp, mKCP, and HTTPUpgrade settings, including TLS and
  REALITY client fields. Unsupported wire-affecting values fail with an
  actionable error instead of producing a partial config.
- **Keep distinct configs distinct.** Canonical v2 deduplication includes the
  preserved extension map, preventing configs with different transport or
  security parameters from collapsing into one record.

### Safer rotation

- Health monitoring distinguishes immediate process/inbound failures from
  data-plane failures. HTTP checks run asynchronously through the active proxy
  and require a configurable number of consecutive failures before recovery.
- Automatic and unpinned rotations use fresh candidate tests. ICMP remains
  diagnostic and cannot qualify a candidate by itself.
- Replacement configs pass the native Xray, V2Ray, or sing-box validator before
  the active runtime stops. If the replacement fails during handoff, xrat tries
  to restore the previous runtime.
- `xrat rotate enable` and `xrat rotate disable` persist their state in
  `config.toml`. Rotation status includes the failure threshold, current
  failure count, probe state, last health error, and pending recovery state.
- The TUI settings modal exposes the new
  `runtime.rotation.health_failure_threshold` option as **Failure threshold**
  with contextual help and validation. The default is `3`.

### Upgrade notes

- Migration `0022` adds `configs.extensions_json` on SQLite and PostgreSQL. It
  runs automatically and backfills legacy v1 config rows from their stored raw
  links while preserving config IDs and refs.
- Existing `config.toml` files remain compatible. The new health threshold uses
  its default until explicitly configured.
- No CLI commands or supported import schemes were removed.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.14.0...v0.15.0
