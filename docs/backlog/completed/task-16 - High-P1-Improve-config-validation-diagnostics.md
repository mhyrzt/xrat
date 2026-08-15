---
id: TASK-16
title: 'High, P1: Improve config validation diagnostics'
status: Done
assignee: []
created_date: '2026-07-05 14:43'
updated_date: '2026-07-11 21:22'
labels:
  - legacy-import
  - improvement
dependencies: []
priority: high
ordinal: 3000
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

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. error.rs: fix ConfigToml thiserror message to interpolate {0} for real toml line/col detail.
2. validate.rs: parse file as toml::Value before AppConfig deserialize; syntax failure becomes structural diagnostic with real toml error.
3. Add helpers: check_scalar_enum, check_string_array_enum, check_duration_field operating on toml::Value subtrees.
4. Wire helpers to testing.order, testing.failure_policy, database.backend, testing.geoip.backend/.fallback, testing.geoip.remote.provider (real enums) and runtime.engine, routing.domain_strategy, dns.query_strategy (plain strings today, add accepted-list checks).
5. Skip full AppConfig deserialize attempt when Value-level diagnostics found; otherwise fall back to existing serde-based validate_config path unchanged.
6. Add validate_geo/validate_routing/validate_dns functions closing current validation gaps.
7. Tests per Verification list: bad order entry, bad failure_policy, string-typed timeout, broken TOML syntax, valid config unchanged output.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented: error.rs ConfigToml now interpolates {0} so the real toml::de::Error (with line/col) surfaces instead of a static message. validate.rs collect_errors now reads file contents once, parses as toml::Value (via toml::from_str::<toml::Value>, not Value::from_str which only parses a single value literal not a document - caught this via a failing test and fixed), and runs check_known_fields against it before attempting AppConfig deserialize; syntax errors get a structural diagnostic with the real toml error, field-level diagnostics short-circuit before the noisier generic serde-based fallback.
Helpers: check_scalar_enum, check_string_array_enum, check_duration_field, operating on toml::Value via get_path; enum accepted-value tables (canonical + aliases) for testing.order, testing.failure_policy, database.backend, testing.geoip.backend/.fallback, testing.geoip.remote.provider.
Scope decision: did NOT add validation for runtime.engine, routing.domain_strategy, dns.query_strategy - these are plain String fields that never block deserialization (unlike the true Rust enums above), so they don't hit the "opaque parse failure" problem this task targets. runtime.engine already has a working post-deserialize check. Adding accepted-value lists for domain_strategy/query_strategy would mean guessing xray vs sing-box accepted values not sourced from the task, so left as a follow-up decision rather than guessed.
Tests added: 8 new tests in validate.rs covering unknown order entry, unknown failure_policy, string-typed duration, unknown database backend, unknown geoip backend, invalid TOML syntax, and a valid-config-on-disk regression check; existing 22 validate.rs tests still pass unchanged (no behavior regression on the semantic post-deserialize path).
cargo fmt/clippy clean. cargo test --locked: 647/647 passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Fixed AppError::ConfigToml to interpolate the real toml::de::Error (line/col) instead of a static message. Added a toml::Value pre-pass in validate.rs's collect_errors that runs before AppConfig deserialization: syntax errors get a structural diagnostic with the real toml error, and new reusable helpers (check_scalar_enum, check_string_array_enum, check_duration_field) report field-level diagnostics for testing.order, testing.failure_policy, database.backend, and GeoIP backend/fallback/provider — the enum-backed fields that previously produced only an opaque generic parse failure. Field-level diagnostics short-circuit before the noisier serde-based fallback. Deliberately left runtime.engine/routing.domain_strategy/dns.query_strategy out of scope: they're plain strings that never block deserialization (not this task's problem), and guessing accepted values across xray/sing-box risked incorrect guidance. Verified with 8 new tests (bad order entry, bad failure_policy, string-typed duration, bad database/geoip backend, invalid TOML syntax, valid-config regression) plus all 22 existing validate.rs tests unchanged. cargo fmt/clippy clean, cargo test --locked: 647/647 passed.
<!-- SECTION:FINAL_SUMMARY:END -->
