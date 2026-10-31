## xrat v0.34.0

This release adds a Linux TUN kill switch and optional transparent TUI
backgrounds.

### Linux TUN kill switch

- Enable persistent kernel egress filtering for managed TUN capture in
  `split_mode = "all"`, using nftables or the iptables/ip6tables fallback.
- Choose `kill_switch = "auto"` to retain firewall rules after an unexpected
  engine exit and remove them on intentional disconnect or TUN disable. Choose
  `"strict"` to retain rules until explicitly turned off or reset.
- Inspect or manage protection with `xrat tun killswitch status`, `on`, `off`,
  and `reset`. Run `xrat tun enable --kill-switch` to enable TUN with auto mode.
- Preserve local network access with `allow_lan = true` (the default), or set it
  to `false` to remove the LAN exception. Rules also permit loopback, TUN
  traffic, marked engine sockets, proxy endpoints, DHCP, and IPv6 neighbor
  discovery; other egress is dropped.
- Reject whitelist/blacklist split tunneling while the kill switch is enabled.
  The firewall implementation is Linux-only; activation on macOS or Windows
  reports unsupported.

Example:

```toml
[runtime.tun]
enabled = true
split_mode = "all"
kill_switch = "auto"
allow_lan = true
```

The runtime owner must have permission to manage firewall rules. If retained
rules block access after a crash, use `xrat tun killswitch reset` with the
necessary privileges. See the
[TUN guide](https://mhyrzt.github.io/xrat/02-cli/tun.html) for configuration and
recovery commands.

### Transparent TUI backgrounds

- Set `[tui].transparent = true`, or enable transparency from the TUI Settings
  dialog, to use the terminal's default background. Terminal-configured opacity
  and blur can then show through.
- Keep theme text, borders, accent colors, and selected-row highlights. Built-in
  and custom themes support the setting.
- With transparency disabled, the ANSI theme uses an opaque black background.

```toml
[tui]
transparent = true
```

### Reliability

- Preserve the attached firewall policy during interrupted iptables swaps and
  failed retries, including IPv4 and IPv6 recovery. Status recognizes an
  attached staging policy.
- Terminate the spawned engine and record a failed session if firewall
  activation fails.
- Clear auto-mode rules after crash reconciliation removes the TUN ownership
  journal. Report removal failures, retain settings, and restore the prior live
  TUN connection when a disable transition fails.
- Preserve strict-mode protection and support retrying failed cleanup.

### Contributors

Thank you to [@f02xygen](https://github.com/f02xygen) for the Linux TUN kill
switch in [#281](https://github.com/mhyrzt/xrat/pull/281) and TUI transparency
in [#282](https://github.com/mhyrzt/xrat/pull/282).

### Upgrade notes

- Kill switch mode defaults to `"off"`; TUI transparency defaults to `false`.
  Existing configuration files remain valid.
- No new database migrations are included.
- Windows support remains experimental. Native automated checks do not replace
  the outstanding interactive Windows and actual TUN traffic acceptance work;
  see the [Windows Support roadmap](https://github.com/mhyrzt/xrat/issues/262).
- Refresh an already open browser tab after upgrading.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.33.0...v0.34.0
