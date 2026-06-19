# 12.02 Windows service install

**Difficulty:** Large, P3.

Windows should run the long-lived daemon through the Service Control Manager.
Scheduled Tasks are a fallback only if service permissions make a first version
impractical.

## Current state

- `src/app/commands/daemon_install.rs` supports systemd user services on Linux,
  launchd user agents on macOS, and rc.d scripts on FreeBSD/OpenBSD.
- Windows falls through to `UnsupportedPlatform` for both install and
  uninstall.
- `xrat setup` can call daemon install, but its daemon path detection only knows
  Linux and macOS.

## Target behavior

- `xrat daemon install` creates a Windows Service that runs:
  `xrat daemon run-server`.
- `--start` starts the service after registration.
- `--dry-run` prints the service name, binary path, arguments, working data
  path, and service-control actions without changing the system.
- `xrat daemon uninstall` stops, disables, and deletes the service while
  preserving xrat config and data.
- `--with-api` follows the existing CLI contract. If a second service is still
  required by the implementation, document and install it consistently with the
  daemon service.

## Implementation notes

- Prefer direct SCM integration through a Rust crate or Windows APIs over
  fragile shell parsing of `sc.exe` output.
- Use stable service names such as `xrat-daemon` and, if needed, `xrat-api`.
- Store service display names and descriptions in code or
  `packaging/windows/` templates.
- Detect permission failures cleanly and tell the user to rerun from an
  elevated shell when required.
- Update `xrat setup` daemon probing so Windows can report installed, missing,
  skipped, or failed states accurately.

## Tests and verification

- Unit-test service command rendering or API parameter construction without
  touching SCM.
- Add Windows runtime checks for install dry-run output.
- On a Windows host, verify install, start, status, stop, restart, uninstall,
  and reinstall.
- Verify uninstall handles missing services without removing user config.

## Completion criteria

- `xrat daemon install --dry-run` and `xrat daemon uninstall --dry-run` work on
  Windows.
- Real install/uninstall work from an elevated shell.
- User docs describe Windows service requirements and permissions.
