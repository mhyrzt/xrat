## xrat v0.12.0

This release improves the terminal proxy workflow (`xrat proxy shell`) and
clarifies how TUI bulk tests relate to the CLI.

### Features

- **Proxy shell protocol selection.** `xrat proxy shell enable` now accepts an
  optional trailing protocol: `http`, `socks5`, or `socks5h`. When given, the
  matching inbound is required and the same scheme is used for
  `http_proxy`/`https_proxy` and `all_proxy`. Without it, the existing
  prefer-HTTP-then-SOCKS behavior is unchanged.
- **Proxy shell status + usage hints.** `enable`, `disable`, and `toggle` now
  print the current proxy shell status to stderr after emitting their script,
  so `eval "$(xrat proxy shell ...)"` output stays clean. Each emitted script
  starts with a `#` comment showing how to apply it for the detected shell
  (bash/zsh `eval`, fish `| source`), and the same usage note appears in each
  subcommand's `--help`.

### Docs

- TUI `t + a` bulk tests derive their stages from
  `[runtime.rotation].test_stages` and always skip TCP/upload; the docs now
  describe this instead of the previous stale "TCP and real-delay" wording.

### Upgrade notes

- No new database migrations; safe drop-in upgrade.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.11.0...v0.12.0
