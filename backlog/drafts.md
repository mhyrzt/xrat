# Drafts

---

## D01. Medium, P1: Docker image bundles proxy engines

### Status

Planned

### Goal

Include pre-installed xray and sing-box binaries in the Docker image so
container users do not need to install or mount them separately.

### Changes required

- Pick a base image strategy: install from distro packages, download release
  archives during the Docker build, or multi-stage copy from upstream images.
- Keep the image size reasonable — prefer release archives or Alpine packages.
- Verify both engines work with the default runtime user inside the container.

### Decisions

- Need to check which architectures the base image targets and whether both xray
  and sing-box publish matching musl/glibc builds.

---

## D02. Medium, P1: PaaS-friendly config via env var

### Status

Planned

### Goal

Allow passing the entire `config.toml` as a base64-encoded environment variable
so xrat can run on PaaS platforms (common in Iran) that do not support mounting
custom config files.

### Changes required

- Add a `--config-b64` CLI flag or `XRAT_CONFIG_B64` env var that decodes and
  uses the value as the config file content.
- Ensure the decoded content lands in the right path before xrat reads it, or
  feed it directly into the config parser so filesystem lookup is optional.
- Document the flag/env var in the PaaS deployment section of the docs.

### Verification

- Unit test: encode a known config, pass it via the flag, confirm xrat reads the
  expected values.
- Integration test: run `xrat init` with `XRAT_CONFIG_B64` set and no config
  file on disk.

---

## D03. Easy, P2: Desktop icon shows in DE taskbar

### Status

Planned

### Goal

When launched from the app menu or search, xrat's window shows the xrat icon in
the DE taskbar/dock instead of the terminal emulator's icon (kitty, alacritty,
etc.).

### Current behavior

`packaging/desktop/xrat.desktop` has `Terminal=true`. DEs launch the Exec
command inside a terminal emulator, and the terminal emulator's window appears
with its own icon in the taskbar — never the xrat icon.

### Options considered

1. **`Terminal=false` + specific terminal**: Change to `Exec=kitty -e xrat tui`
   with `Terminal=false`. Still shows the terminal icon because the window
   belongs to kitty.
2. **Wrapper script with `StartupWMClass`**: Launch from a script, set
   `StartupWMClass=xrat` in the desktop entry, and configure the terminal to set
   WM_CLASS to `xrat`. Fragile and terminal-dependent.
3. **`Terminal=false` only**: Remove `Terminal=true`. The TUI runs in the
   foreground; the user is responsible for launching from a terminal. The
   desktop entry acts as a quick-launch shim that opens a terminal window. This
   still does not fix the icon.
4. **Accept the limitation**: Document that `Terminal=true` entries inherently
   show the terminal icon and this is a DE constraint, not a xrat bug.

### Changes required

- Pick an approach (likely option 2 or 4) and document the reasoning.
- If option 2: write a small shell wrapper under `packaging/desktop/` that
  spawns a terminal with the right WM_CLASS, and reference it from the desktop
  entry.
- Ensure `StartupNotify=true` is set so the DE shows a launch-feedback cursor.

### Verification

- Launch from app menu, confirm the dock/taskbar icon is the xrat icon (or, if
  option 4, confirm the limitation is documented and the terminal icon behavior
  is expected).

---

- xrat daemon restart reattach previous connection or connect to best connection
- show a message when config copied at clipboard at chromebar center with
  following message `copied config#ref to clipboard` and remove it after some
  time
- Logs > xrat event can have more standard spacing and be more aligned currently
  is like this

```
2026-06-07 00:15:28  info   runtime       daemon_restart_stale_pid_recovered  Reconnected config 2 after stale runtime PID on daemon start
2026-06-07 00:15:27  info   daemon        daemon_started  Daemon supervisor started

```
