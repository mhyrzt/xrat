# Medium, P2: Explain invalid config diagnostics in `xrat validate`

### Status

Planned

### Goal

Make `xrat validate` explain why each config value is invalid and include
actionable ways to fix it. Validation output should help users repair
`config.toml` without reading source code or guessing valid ranges.

### Current behavior

- `xrat validate <path>` reports validation failures from
  `src/app/commands/validate.rs`.
- Existing messages identify the invalid field and condition, but they do not
  consistently explain the reason, accepted values, or likely fixes.
- JSON output exposes the same error strings as human output, so structured
  consumers also lack repair guidance.

### Changes required

- Replace plain validation strings with a small diagnostic type containing:
  - field/path, such as `[runtime.socks].port`
  - what is invalid
  - why it matters
  - possible solution(s)
- Render human output with compact, readable diagnostics for each invalid field.
- Render JSON output with stable fields so tools can consume diagnostics without
  parsing prose.
- Keep parse/deserialization errors useful by wrapping them with context when
  possible, especially for invalid enum values, malformed TOML, missing required
  secrets, and invalid URLs.
- Include accepted values or ranges in diagnostics for port numbers, timeouts,
  URL schemes, database settings, server bind/key settings, and test settings.
- Keep the success output unchanged unless a better diagnostics structure
  requires a compatible JSON envelope change.

### Verification

- Unit tests for invalid config cases assert the field, reason, and suggested
  fix are present.
- CLI output tests cover human and JSON formats.
- Manual:
  - Run `xrat validate` against a config with several invalid fields.
  - Confirm output explains why each value is invalid and how to fix it.

### Open decisions

- Whether JSON output should remain a list of strings for compatibility or move
  to a versioned object containing structured diagnostics.
- Whether multiple suggestions should be ranked or emitted as an unordered list.
