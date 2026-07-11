## xrat v0.10.0

This release adds TCP as a standalone test stage, sharper config validation
diagnostics, new Xray runtime tuning options, and clearer dial-endpoint GeoIP
reporting.

### Features

- **TCP as a first-class test stage.** `[testing].order` and rotation
  `test_stages` now accept `tcp` directly instead of only running it as an
  implicit gate before `real_delay`. TCP-only pipelines are now possible
  (`order = ["icmp", "tcp"]`), rotation can select candidates by TCP latency
  alone, and stages dedupe automatically when both `tcp` and `real_delay` are
  present.
- **Field-level config validation diagnostics.** `xrat validate` now checks
  the raw TOML value before deserializing, so an invalid enum value or
  wrong-typed field (`[testing].order`, `failure_policy`, `[database].backend`,
  GeoIP backend/provider settings, duration fields) is reported against the
  specific field with accepted values, instead of a generic parse failure.
- **Xray runtime tuning.** New `[runtime.mux]`, `[runtime.fragment]`, and
  `[runtime.network]` config sections: client-side Mux multiplexing, TCP
  fragmentation of the TLS ClientHello, and interface/source/mark binding for
  generated Xray outbounds. All opt-in and disabled by default.
- **Dial-endpoint GeoIP columns.** `xrat test` output gains COUNTRY and
  FRONTING columns (plus the full `dial_endpoint_*` set in tsv/csv), resolved
  per config independently of the real-delay stage, with CDN/relay fronting
  detection and lookup-source provenance.
- **Grouped test failures.** The `xrat test` failures footer now collapses
  identical failure reasons into one line with the affected config refs
  listed beneath, instead of one line per config.
- **Animated TUI demo.** The README preview is now an animated GIF instead of
  a static screenshot.

### Upgrade notes

- New migration `0021_rename_endpoint_to_dial_endpoint`: renames the
  persisted `endpoint_*` GeoIP columns to `dial_endpoint_*`. Applied
  automatically on next run.
- Breaking for JSON API consumers: `xrat validate --format json` now emits
  `errors` as objects (`field`, `problem`, `reason`, `fix`) instead of plain
  strings.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.9.1...v0.10.0
