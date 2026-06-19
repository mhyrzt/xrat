# 12.06 Windows runtime verification

**Difficulty:** Medium, P3.

Some Windows behavior is likely already portable, but it should not be marked
supported until it is tested on Windows with the real runtime engines.

## Current state

- Process reattach uses `sysinfo`, which has Windows support.
- TUI clipboard uses `arboard`, which has a Windows backend.
- ICMP command flags include a Windows branch.
- Path helpers include Windows parent-process support, but many setup and
  desktop integration paths are still Unix or Linux/XDG focused.

## Target behavior

- xray and sing-box can be discovered, spawned, stopped, and reattached on
  Windows.
- Runtime session state remains accurate after CLI restart and daemon restart.
- Clipboard copy actions work in the TUI.
- Setup reports Windows-specific skipped, missing, done, and failed states
  truthfully.
- User docs and platform matrix reflect the verified Windows behavior.

## Verification scenarios

- Build and test:
  - `cargo test --locked` on `windows-latest`
  - `cargo clippy --all-targets -- -D warnings` on Windows if CI time permits
- Runtime:
  - `xrat init`
  - `xrat import <input>`
  - `xrat parse <config-id>`
  - `xrat test <config-id>`
  - `xrat connect <config-id>`
  - close the CLI, then verify `xrat status` reattaches to the running engine
  - `xrat disconnect`
- Daemon:
  - `xrat daemon start`
  - `xrat daemon status`
  - `xrat connect <config-id>` through daemon control
  - `xrat rotate status`
  - `xrat daemon stop`
- UI and proxy:
  - TUI opens and renders without terminal escape issues in Windows Terminal
  - TUI clipboard actions copy expected text
  - desktop proxy manual and PAC modes work after implementation

## Completion criteria

- A Windows host or CI runner has executed the acceptance scenarios.
- Any Windows-only caveats are documented in `docs/src/`.
- Windows can be moved from deferred to supported in the platform matrix.
