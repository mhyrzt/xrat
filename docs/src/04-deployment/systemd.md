# systemd Services

Run xrat as a systemd user service for persistent operation and automatic
startup on login. To start at boot before login, enable systemd user lingering
as shown below.

systemd user services run under your user account (not root) and are managed
with `systemctl --user`.

Benefits:

- **Auto-start**: Service starts on login
- **Restart on failure**: Automatically restarts if the process crashes
- **Logging**: Integrated with `journalctl`

## Installation

Use `xrat daemon install` to generate and enable the service automatically. This
is the recommended approach — no manual file editing required.

```bash
xrat daemon install
```

To also start the daemon immediately:

```bash
xrat daemon install --start
```

To install the standalone HTTP API service alongside the daemon:

```bash
xrat daemon install --with-api
```

To preview what would be written without making changes:

```bash
xrat daemon install --dry-run
```

The command:

1. Resolves the current binary path
2. Generates `xrat-daemon.service` with the correct `ExecStart` and `XRAT_PATH`
3. Writes to `~/.config/systemd/user/` (respects `$XDG_CONFIG_HOME`)
4. Runs `systemctl --user daemon-reload`
5. Runs `systemctl --user enable xrat-daemon.service`

## Removal

```bash
xrat daemon uninstall
```

Stops, disables, and removes the service file. All user config, database, logs,
and application state are preserved.

Preview first:

```bash
xrat daemon uninstall --dry-run
```

## Management

```bash
systemctl --user start xrat-daemon
systemctl --user stop xrat-daemon
systemctl --user restart xrat-daemon
systemctl --user status xrat-daemon
```

View logs:

```bash
journalctl --user -u xrat-daemon -f
journalctl --user -u xrat-daemon --since today
journalctl --user -u xrat-daemon -n 100
```

## Lingering

By default, user services start with your login session and stop when you log
out. To let the user service manager start at boot before login, and to keep the
daemon running without an active login session:

```bash
loginctl enable-linger $USER
```

To undo this:

```bash
loginctl disable-linger $USER
```

## Environment Variables

The generated service file sets `XRAT_PATH` and `RUST_LOG=info`. To add secrets
(API key, passwords), use an environment file.

Create `~/.config/xrat/env`:

```bash
XRAT_API_KEY=your-secret-key
XRAT_POSTGRES_PASSWORD=your-db-password
```

Then add to the service unit after running `daemon install`:

```ini
[Service]
EnvironmentFile=%h/.config/xrat/env
```

## Troubleshooting

**Service won't start**:

```bash
systemctl --user status xrat-daemon
journalctl --user -u xrat-daemon -n 50
```

Common causes: binary not found (check `ExecStart` path), port already in use,
config parse error (test with `xrat daemon start` manually).

**Service stops unexpectedly**:

```bash
journalctl --user -u xrat-daemon --since "1 hour ago"
```

**Logs not appearing**: ensure `Environment=RUST_LOG=info` is set in the unit.

---

## Reference: Manual Setup

If you cannot use `xrat daemon install` (e.g., the binary is not yet in PATH),
you can create the service file manually.

```bash
mkdir -p ~/.config/systemd/user
```

**`~/.config/systemd/user/xrat-daemon.service`**:

```ini
[Unit]
Description=XRAT Daemon
After=network.target

[Service]
Type=simple
ExecStart=/path/to/xrat daemon run-server
Restart=on-failure
RestartSec=5

Environment=XRAT_PATH=/home/user/.config/xrat
Environment=XRAT_API_KEY=
Environment=RUST_LOG=info

NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/home/user/.config/xrat
PrivateTmp=true

[Install]
WantedBy=default.target
```

Replace `/path/to/xrat` with the actual binary location. Then:

```bash
systemctl --user daemon-reload
systemctl --user enable xrat-daemon.service
systemctl --user start xrat-daemon.service
```

The template files used by `xrat daemon install` are available in the repository
at `packaging/systemd/`.

---

## Related

- [`daemon`](../02-cli/daemon.md) — daemon CLI reference including
  install/uninstall
- [Deployment](index.html) — deployment overview
- [HTTP API](../03-features/http-api.md) — API server details
- [Daemon and IPC](../03-features/daemon-and-ipc.md) — daemon supervisor
  internals
