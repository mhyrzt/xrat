## xrat v0.18.0

This release applies configured DNS behavior to managed proxy runtimes and Xray
connection probes, exposes DNS settings in the TUI, and makes managed core
downloads visibly progress.

### Managed DNS

- **Apply DNS settings to every managed engine.** Xray and V2Ray receive the
  complete configured DNS object. sing-box receives validated modern typed DNS
  servers, supported query strategies, cache settings, and exact host mappings.
- **Fail safely on unsupported sing-box mappings.** Strategies, server forms,
  host patterns, and fallback options without faithful modern sing-box
  equivalents are rejected before the managed process starts instead of being
  silently ignored.
- **Keep defaults minimal.** Runtime DNS blocks are omitted when the DNS section
  remains at its defaults, preserving engine-native behavior for unchanged
  installations.

### Testing and configuration

- Xray probe configs used by real-delay, download, and upload tests now include
  non-default `[dns]` settings. Direct ICMP and TCP checks remain unaffected.
- The TUI settings editor now exposes the existing DNS fields, including server
  lists and static host mappings, with the same validation used by file-based
  configuration.
- The generated default configuration and user documentation now distinguish
  the complete Xray/V2Ray support from sing-box's validated support subset.

### Setup feedback

- Interactive managed-core downloads now show byte progress, so larger Xray,
  sing-box, and V2Ray archives no longer make `xrat setup` appear frozen.
- Redirected and machine-readable setup flows suppress progress rendering to
  keep their output stable.

### Upgrade notes

- No database migration is required, and existing `config.toml` files remain
  syntactically compatible.
- Existing non-default `[dns]` settings now affect managed runtime and Xray
  probe generation. Review them before upgrading if they were previously kept
  only as documentation or future configuration.
- Custom sing-box DNS requires `UseIPv4` or `UseIPv6` and supported server/host
  forms. Unsupported values now produce an actionable pre-launch error; use the
  documented support matrix when adapting an existing configuration.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.17.0...v0.18.0
