## xrat v0.18.1

This patch release fixes VLESS XHTTP links that use extended transport
parameters and prevents supported-looking Xray link options from being silently
dropped during runtime config generation.

### XHTTP compatibility

- **Generate official XHTTP extras.** JSON supplied through the `extra` query
  parameter is decoded and emitted as `xhttpSettings.extra`, preserving nested
  options supported by newer Xray-core versions.
- **Support common padding spellings.** `xPaddingBytes`, `x_padding_bytes`, and
  the URL-encoded `x_padding%20bytes` alias all generate the canonical
  `xPaddingBytes` field.
- **Map current flat fields safely.** Supported XHTTP strings, booleans,
  integers, headers, xmux settings, and download settings are validated and
  merged with documented precedence.

### Safer link handling

- Xray link parameters are accepted only when the relevant protocol, security,
  or transport builder consumes them with the expected type.
- Unknown flat parameters, malformed JSON, repeated singular values,
  conflicting aliases, and parameters used with the wrong transport now return
  actionable errors instead of producing incomplete runtime configs.
- Future XHTTP fields can be carried inside the typed JSON `extra` object. Their
  runtime availability still depends on the installed Xray-core version.

### Upgrade notes

- No database migration or configuration-file change is required.
- Existing XHTTP links using the reported padding aliases should work without
  modification.
- Put newly introduced XHTTP options inside the URL-encoded JSON `extra`
  parameter when xrat does not yet recognize their flat form.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.18.0...v0.18.1
