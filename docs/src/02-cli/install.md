# install

Install a managed proxy core directly from its upstream GitHub releases.

```text
xrat install <CORE> [--version <VERSION> | --prerelease]
```

Supported cores are `xray`, `v2ray`, and `sing-box` (`singbox` is also accepted).
Without `--version`, XRAT installs the latest stable release reported by GitHub.
Pass `--prerelease` to install the newest published prerelease instead.

```bash
# Install the latest Xray release
xrat install xray

# Install a specific V2Ray release
xrat install v2ray --version 5.52.0

# A leading v in the version is accepted
xrat install sing-box --version v1.13.2

# Install the newest published Xray prerelease
xrat install xray --prerelease
```

XRAT downloads the platform-specific asset from the core's official repository,
verifies its published SHA-256 digest, stages and validates the binary, and then
atomically installs it under the user data directory. It also updates the matching
runtime binary path in `config.toml` and creates a command link in `~/.local/bin`
when that location is available. Existing non-XRAT command links are left untouched.

The download is streamed with a progress bar in interactive terminals. Completion
output reports the installed core, version, binary path, and updated `config.toml`
path. The same details are recorded as a best-effort `core_installed` event for
`xrat logs`.

Managed installation currently supports Linux and macOS on x86-64 and ARM64.
