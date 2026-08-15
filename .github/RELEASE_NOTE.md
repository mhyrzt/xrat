## xrat v0.17.0

This release adds verified user-local installation and upgrades for Xray,
sing-box, and V2Ray, and restores native preflight validation with Xray 26.3.27.

### Managed proxy cores

- **Install without root access.** `xrat setup` can install Xray, sing-box, and
  V2Ray under `~/.local/share/xrat/cores` and expose safe CLI links through
  `~/.local/bin`.
- **Use verified official releases.** Setup discovers the latest stable release
  from each core's official GitHub repository, requires its published SHA-256
  digest, verifies the staged binary version, and replaces managed copies
  atomically. A failed update leaves the previous core usable.
- **Preserve system packages.** Existing external or package-managed binaries
  are detected and never overwritten. Accepting an update installs an isolated
  managed copy and records its path in `config.toml`.
- **Keep assets isolated.** Managed Xray and V2Ray processes receive their own
  GeoIP and Geosite asset directories during validation, testing, and runtime
  startup.

### Setup behavior

- `xrat setup --check` reports missing, current, and outdated cores. An
  `update_available` result is visible in table and JSON output without making
  an otherwise healthy check fail.
- `xrat setup --yes` installs missing Xray and sing-box, skips an absent V2Ray,
  and upgrades installed outdated cores.
- Release lookup failures do not disable an installed core; setup reports that
  the update check failed and keeps the detected binary available.
- The piped `install.sh` flow reconnects setup prompts to the controlling
  terminal. When no terminal is available, it prints a clear unattended
  fallback instead of silently consuming closed standard input.

### Xray compatibility

- Native runtime preflight files now retain a `.json` suffix. Xray 26.3.27 uses
  the filename to select its config parser and previously rejected the
  extensionless temporary files with exit status 23, preventing connection and
  rotation even when the generated configuration was valid.

### Upgrade notes

- No database migration is required, and existing `config.toml` files remain
  compatible.
- Run `xrat setup --check` to inspect installed core versions. Run `xrat setup`
  to adopt managed copies interactively.
- Existing external Xray, sing-box, and V2Ray installations remain selected
  until an installation or update is explicitly accepted.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.16.1...v0.17.0
