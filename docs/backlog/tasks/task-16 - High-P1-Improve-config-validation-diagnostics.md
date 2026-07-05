---
id: TASK-16
title: 'High, P1: Improve config validation diagnostics'
status: To Do
assignee: []
created_date: '2026-07-05 14:43'
labels:
  - legacy-import
  - improvement
dependencies: []
priority: high
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/config-validate-diagnostics.md`

# High, P1: Improve config validation diagnostics

### Status

Planned

### Motivation

`xrat validate ~/.config/xrat/config.toml` currently reports some malformed
configuration as a broad parse failure:

```text
config failed to parse: failed to parse application config
  why: the file is not valid TOML or has a field with the wrong type or an unknown enum value.
```

That message is technically accurate, but it is not actionable enough. A user
who sets `[testing].order = ["icmp", "tcp"]` should see that the invalid value
is specifically in `[testing].order`, what value was rejected, and which values
are accepted.

### Current behavior

- Semantic validation in `src/app/commands/validate.rs` can produce good
  field-level diagnostics, but it only runs after `AppConfig` deserializes.
- Serde enum failures, wrong field types, and unknown enum values can stop
  parsing before `validate` can identify the exact field.
- The parser-level error groups several different problems together: invalid
  TOML syntax, wrong type, and unknown enum value.
- This is especially confusing for enum-backed fields such as:
  - `[testing].order`
  - `[testing].failure_policy`
  - `[runtime].engine`
  - `[database].backend`
  - GeoIP backend/provider settings
  - routing and DNS strategy settings

### Desired behavior

Validation should distinguish syntax errors from schema/value errors, and schema
errors should point to the field whenever possible:

```text
error: /home/user/.config/xrat/config.toml is invalid

  [testing].order
    value "tcp" is not accepted here
    accepted values: icmp, real_delay, download
    note: TCP is currently controlled by [testing.tcp].enabled and runs before real_delay
```

For wrong types, prefer messages like:

```text
[testing.tcp].timeout
  expected integer milliseconds, got string
```

### Changes required

- Consider parsing the TOML into `toml::Value` first, then validating known
  fields before or alongside deserializing into `AppConfig`.
- Add field-specific checks for enum arrays and scalar enums so invalid values
  can be reported without relying on serde's generic parse error.
- Keep serde deserialization as the source of truth for building `AppConfig`,
  but convert common deserialization failures into field-level diagnostics.
- Add helper functions for reusable enum diagnostics:
  - scalar enum field: value, accepted values, aliases
  - string array enum field: invalid entries, duplicates, accepted values
  - integer duration fields: wrong type, zero when enabled, negative if present
- Make the final parse failure include the underlying parser/deserializer error
  only as supporting detail, after actionable diagnostics.
- Ensure diagnostics remain stable enough for tests without over-coupling to
  exact third-party error wording.

### Verification

- `order = ["icmp", "tcp"]` reports `[testing].order` with the rejected value.
- `failure_policy = "skip"` reports `[testing].failure_policy` with accepted
  values and aliases.
- `timeout = "2000"` under `[testing.tcp]` reports an expected integer.
- Invalid TOML syntax still reports a syntax/location error.
- Existing successful validation output remains unchanged:
  `OK <path> is valid.`

### Open decisions

- Should validation report all detectable schema errors from the raw TOML value,
  or stop after the first parse/deserialization blocker?
- Should aliases be shown separately from canonical accepted values?
- Should `xrat init` comments be updated to mention common validation mistakes,
  or should the validator alone carry that guidance?
<!-- SECTION:DESCRIPTION:END -->
