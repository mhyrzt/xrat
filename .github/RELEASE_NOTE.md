## xrat v0.18.2

This patch release restores XHTTP and other non-RAW transports on older
Xray-core versions and adds an explicit command for installing managed proxy
cores from their official releases.

### Xray transport compatibility

- **Support both transport selector names.** Generated `streamSettings` now
  carries identical `network` and `method` values. Older Xray-core versions use
  `network`, while newer versions can use the renamed `method` field.
- **Prevent silent RAW fallback.** Xray versions that do not recognize `method`
  can now select XHTTP, WebSocket, gRPC, mKCP, and HTTPUpgrade correctly instead
  of ignoring the transport selector and defaulting to RAW.
- **Expand regression coverage.** Generated transport selectors are checked
  across RAW and representative non-RAW configurations, including native XHTTP
  validation with Xray-core.

### Managed proxy-core installation

- **Install cores explicitly.** Use `xrat install xray`, `xrat install v2ray`,
  or `xrat install sing-box` to install a managed core from its official GitHub
  releases.
- **Choose a release.** Installation defaults to the latest stable release,
  accepts an exact version through `--version`, and supports the newest
  published prerelease through `--prerelease`.
- **Verify and persist installs.** Downloads show interactive progress, verify
  the published SHA-256 digest, validate and atomically install the binary, and
  update the matching path in `config.toml`.
- **Improve observability.** Completion output and `xrat logs` identify the
  selected core, installed version, binary path, and updated configuration path.

### Upgrade notes

- No database migration or manual configuration change is required.
- Users running XHTTP or another non-RAW transport with an older Xray-core
  version should upgrade XRAT to avoid the silent RAW fallback.
- Managed core installation supports Linux and macOS on x86-64 and ARM64.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.18.1...v0.18.2
