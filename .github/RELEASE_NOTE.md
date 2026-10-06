## xrat v0.22.0

This release adds opt-in managed TUN capture for Linux with Xray and sing-box,
with privilege diagnostics and safer runtime handoffs.

### Features

- Configure system traffic capture through `[runtime.tun]`, including interface
  addresses, routing, MTU, and engine-specific settings.
- Use `xrat tun setup` to grant required file capabilities and configure the
  systemd user-service override. `xrat tun status` reports engine, xrat, service,
  and effective daemon privilege readiness.
- Follow the new README TUN walkthrough for setup, connection, and returning to
  per-app proxying.

### Fixes and maintenance

- Validate engine capabilities, native configuration, and interface ownership
  before replacing a healthy runtime. Cleanup requires a verified matching
  kernel interface index; failed handoffs attempt to restore the previous config.
- Generate Xray capture routes for the configured IPv4/IPv6 address families.
- Accept the client-specific `support-x25519mlkem768` share-link parameter.
- Move project planning to GitHub issues and milestones, preserving completed
  and archived records, original metadata, and task relationships.

### Upgrade notes

- TUN stays disabled by default. Keep at least one local inbound enabled.
- Linux TUN requires network privileges and `libcap` tools. Run `xrat tun setup`
  after upgrading xrat or a managed engine, then restart the daemon.
- Xray TUN requires version `26.7.28` or newer with working Linux TUN support;
  older cores are rejected. V2Ray TUN is unsupported.
- TUN does not intercept DNS. System resolver queries can bypass the tunnel;
  capture routes follow the configured address families.
- No database migration is required for this release.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.21.2...v0.22.0
