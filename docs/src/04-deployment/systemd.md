# systemd Services

Run xrat as a systemd user service for persistent operation and automatic
startup on boot.

## Overview

systemd user services run under your user account (not root) and are managed
with `systemctl --user`.

Benefits:
- **Auto-start**: Service starts on login
- **Restart on failure**: Automatically restarts if the process crashes
- **Logging**: Integrated with `journalctl` for log management
- **Dependency management**: Can depend on network availability

## Service Files

Create service files in `~/.config/systemd/user/`:

```bash
mkdir -p ~/.config/systemd/user
```

### Daemon Service

Runs the xrat daemon with IPC and optional HTTP API.

**File**: `~/.config/systemd/user/xrat-daemon.service`

```ini
[Unit]
Description=xrat Daemon Supervisor
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/xrat daemon start
Restart=on-failure
RestartSec=5

# Environment
Environment=RUST_LOG=info
Environment=XRAT_API_KEY=your-secret-key
Environment=XRAT_SOCKS_PASSWORD=your-socks-password

# Resource limits
LimitNOFILE=65536

[Install]
WantedBy=default.target
```

### HTTP API Service (Standalone)

Runs only the HTTP API server (without daemon).

**File**: `~/.config/systemd/user/xrat-api.service`

```ini
[Unit]
Description=xrat HTTP API
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/xrat serve
Restart=on-failure
RestartSec=5

# Environment
Environment=RUST_LOG=info
Environment=XRAT_API_KEY=your-secret-key

[Install]
WantedBy=default.target
```

### Combined Service (Daemon + API)

Runs the daemon with HTTP API enabled via config.toml.

**File**: `~/.config/systemd/user/xrat.service`

```ini
[Unit]
Description=xrat Daemon with HTTP API
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/xrat daemon start
Restart=on-failure
RestartSec=5

# Environment
Environment=RUST_LOG=info
EnvironmentFile=%h/.config/xrat/env

# Resource limits
LimitNOFILE=65536

[Install]
WantedBy=default.target
```

**Environment file**: `~/.config/xrat/env`

```bash
XRAT_API_KEY=your-secret-key
XRAT_SOCKS_PASSWORD=your-socks-password
XRAT_SHADOWSOCKS_PASSWORD=your-ss-password
XRAT_POSTGRES_USER=xrat
XRAT_POSTGRES_PASSWORD=your-db-password
```

## Installation

### 1. Create Service File

```bash
cat > ~/.config/systemd/user/xrat-daemon.service <<'EOF'
[Unit]
Description=xrat Daemon Supervisor
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/xrat daemon start
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=info
Environment=XRAT_API_KEY=your-secret-key
LimitNOFILE=65536

[Install]
WantedBy=default.target
EOF
```

### 2. Reload systemd

```bash
systemctl --user daemon-reload
```

### 3. Enable Service

Enable to start on login:

```bash
systemctl --user enable xrat-daemon
```

### 4. Start Service

```bash
systemctl --user start xrat-daemon
```

### 5. Check Status

```bash
systemctl --user status xrat-daemon
```

## Management Commands

### Start/Stop/Restart

```bash
systemctl --user start xrat-daemon
systemctl --user stop xrat-daemon
systemctl --user restart xrat-daemon
```

### Enable/Disable Auto-Start

```bash
systemctl --user enable xrat-daemon   # start on login
systemctl --user disable xrat-daemon  # don't start on login
```

### View Logs

```bash
journalctl --user -u xrat-daemon -f          # follow logs
journalctl --user -u xrat-daemon --since today  # today's logs
journalctl --user -u xrat-daemon -n 100      # last 100 lines
```

### Check Status

```bash
systemctl --user status xrat-daemon
```

Output:

```
● xrat-daemon.service - xrat Daemon Supervisor
     Loaded: loaded (/home/user/.config/systemd/user/xrat-daemon.service; enabled)
     Active: active (running) since Thu 2026-05-28 10:30:00 UTC; 2h ago
   Main PID: 12345 (xrat)
      Tasks: 5 (limit: 65536)
     Memory: 15.2M
        CPU: 1.234s
     CGroup: /user.slice/user-1000.slice/user@1000.service/app.slice/xrat-daemon.service
             └─12345 /usr/local/bin/xrat daemon start
```

## Environment Variables

### Inline in Service File

```ini
[Service]
Environment=RUST_LOG=info
Environment=XRAT_API_KEY=secret
```

### Environment File

Create `~/.config/xrat/env`:

```bash
XRAT_API_KEY=your-secret-key
XRAT_SOCKS_PASSWORD=your-socks-password
XRAT_POSTGRES_USER=xrat
XRAT_POSTGRES_PASSWORD=your-db-password
```

Reference in service file:

```ini
[Service]
EnvironmentFile=%h/.config/xrat/env
```

`%h` expands to the user's home directory.

## Lingering

By default, user services stop when you log out. To keep services running:

```bash
loginctl enable-linger $USER
```

This allows services to run even when not logged in (useful for servers).

Check lingering status:

```bash
loginctl show-user $USER | grep Linger
```

## Multi-User Setup

For system-wide services (runs as root or a dedicated user):

**File**: `/etc/systemd/system/xrat.service`

```ini
[Unit]
Description=xrat Daemon
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=xrat
Group=xrat
ExecStart=/usr/local/bin/xrat daemon start
Restart=on-failure
RestartSec=5
EnvironmentFile=/etc/xrat/env

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/xrat
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

Create dedicated user:

```bash
sudo useradd -r -s /bin/false xrat
sudo mkdir -p /var/lib/xrat
sudo chown xrat:xrat /var/lib/xrat
```

Manage with system systemctl:

```bash
sudo systemctl enable xrat
sudo systemctl start xrat
sudo systemctl status xrat
```

## Reverse Proxy

Expose the HTTP API via nginx or Caddy for HTTPS and authentication.

### nginx

```nginx
server {
    listen 443 ssl http2;
    server_name xrat.example.com;

    ssl_certificate /etc/letsencrypt/live/xrat.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/xrat.example.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Caddy

```caddyfile
xrat.example.com {
    reverse_proxy localhost:8080
}
```

## Troubleshooting

### Service Won't Start

**Check**:
```bash
systemctl --user status xrat-daemon
journalctl --user -u xrat-daemon -n 50
```

**Common issues**:
- Binary not found: Check `ExecStart` path
- Permission denied: Check file permissions
- Port already in use: Check if another service is using the port

### Service Stops Unexpectedly

**Check**:
```bash
journalctl --user -u xrat-daemon --since "1 hour ago"
```

**Common issues**:
- Out of memory: Check `LimitNOFILE` and system resources
- Network unavailable: Ensure `After=network-online.target`
- Configuration error: Test config manually with `xrat daemon start`

### Logs Not Appearing

**Check**:
```bash
journalctl --user -u xrat-daemon
```

If empty, ensure logging is enabled:

```ini
[Service]
Environment=RUST_LOG=info
```

## Related

- [Deployment](README.md) — deployment overview
- [HTTP API](../03-features/http-api.md) — API server details
- [Daemon and IPC](../03-features/daemon-and-ipc.md) — daemon supervisor
