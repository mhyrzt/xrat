## xrat v0.19.0

This release aligns Xray parsing and runtime generation with Xray-core v26.3.27
and v26.7.28, while adding the current VMess AEAD share-link format.

### Xray schema compatibility

- **Version-aware generation.** Runtime and probe configs target the installed
  Xray schema automatically. Set `runtime.xray_compatibility` to `stable` or
  `prerelease` to override detection.
- **Audited protocol schemas.** Core, DNS, routing, inbound, outbound,
  transport, TLS, REALITY, mKCP, and socket-option objects now follow the
  audited Xray-core tags, including compatibility aliases and exact acronym
  spelling.
- **Honest parser modes.** Strict parsing rejects unknown nested fields with
  precise paths. Loose parsing preserves unknown root and nested fields when
  configurations are serialized again.
- **Explicit version boundaries.** Removed or version-specific mKCP and TLS
  fields return targeted compatibility errors instead of producing incomplete
  runtime configurations.

### Share links

- **VMess AEAD URLs.** Standard
  `vmess://uuid@host:port?...` links are supported with required UUID, host,
  and port validation. Legacy base64 VMess JSON remains supported.
- **Current security fields.** VMess and VLESS links support `encryption`, TLS
  `ech`/`pcs`/`vcn`, REALITY `pqv`, and percent-encoded finalmask `fm` JSON.
- **Duplicate protection.** URL-form VMess and VLESS links reject repeated
  query parameters as required by the official share-link proposal.

### Validation

- Generated compatibility fixtures were accepted by native Xray-core v26.3.27
  and v26.7.28 binaries.
- No database migration is required.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.18.3...v0.19.0
