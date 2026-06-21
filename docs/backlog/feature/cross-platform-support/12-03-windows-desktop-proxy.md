# 12.03 Windows desktop proxy

**Difficulty:** Medium, P3.

Windows desktop proxy support should target per-user WinINET settings because
that is what browsers and desktop applications commonly read. WinHTTP-only
configuration is not enough for `xrat proxy desktop`.

## Current state

- `src/app/commands/proxy/desktop.rs` implements GNOME through `gsettings` and
  macOS through `networksetup`.
- Non-Linux and non-macOS platforms return `UnsupportedPlatform`.
- Existing actions are `enable`, `disable`, `status`, and `toggle`; `enable`
  and `toggle` can use PAC mode.

## Target behavior

- `xrat proxy desktop enable` sets the current user's WinINET proxy settings
  using active xrat HTTP/SOCKS endpoints.
- `xrat proxy desktop enable --pac` sets `AutoConfigURL` to the xrat PAC URL.
- `disable` clears xrat-managed manual and PAC settings.
- `status` reports enough state to tell whether the user has no proxy, a manual
  proxy, a PAC URL, or settings that xrat should not overwrite blindly.
- `toggle` enables xrat settings when currently disabled and disables them when
  currently xrat-managed.

## Implementation notes

- Use the registry path:
  `HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings`.
- Handle at least `ProxyEnable`, `ProxyServer`, and `AutoConfigURL`.
- Refresh desktop consumers after changes with the WinINET
  `InternetSetOption` settings-changed and refresh calls.
- Prefer preserving unrelated user proxy settings unless the command is clearly
  disabling settings that xrat previously applied.
- Avoid `netsh winhttp set proxy` as the primary implementation because it
  configures WinHTTP rather than the interactive desktop proxy path.

## Tests and verification

- Unit-test registry value planning with a fake settings backend.
- Verify manual proxy enable/status/disable on Windows.
- Verify PAC enable/status/disable against a running xrat API server.
- Verify toggle does not erase unrelated user proxy settings unexpectedly.

## Completion criteria

- `xrat proxy desktop` no longer returns `UnsupportedPlatform` on Windows.
- Manual and PAC modes are visible in status output.
- User docs explain that Windows desktop proxy support uses per-user settings.
