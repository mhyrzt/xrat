# XRAT Systemd Service Setup

This directory contains systemd service files for running XRAT as a background
service.

## Service Files

- `xrat-daemon.service` - Runs the XRAT daemon with supervisor and optional HTTP
  API
- `xrat-api.service` - Runs the standalone HTTP API server (if not using daemon
  mode)

## Installation (User Service)

### Option 1: Daemon Mode (Recommended)

The daemon mode includes the supervisor, auto-rotation, and optionally the HTTP
API.

```bash
# Copy service file
mkdir -p ~/.config/systemd/user/
cp xrat-daemon.service ~/.config/systemd/user/

# Edit to set your API key (optional)
nano ~/.config/systemd/user/xrat-daemon.service
# Set: Environment=XRAT_API_KEY=your-secret-key

# Enable and start
systemctl --user daemon-reload
systemctl --user enable --now xrat-daemon.service

# Check status
systemctl --user status xrat-daemon.service

# View logs
journalctl --user -u xrat-daemon.service -f
```

### Option 2: Standalone API Server

Use this if you only want the HTTP API without the daemon supervisor.

```bash
# Copy service file
mkdir -p ~/.config/systemd/user/
cp xrat-api.service ~/.config/systemd/user/

# Edit to set your API key (optional)
nano ~/.config/systemd/user/xrat-api.service
# Set: Environment=XRAT_API_KEY=your-secret-key

# Enable and start
systemctl --user daemon-reload
systemctl --user enable --now xrat-api.service

# Check status
systemctl --user status xrat-api.service

# View logs
journalctl --user -u xrat-api.service -f
```

## Configuration

### Environment Variables

Both services support these environment variables:

- `XRAT_PATH` - Configuration directory (default: `~/.config/xrat`)
- `XRAT_API_KEY` - API authentication key (optional, recommended for production)
- `RUST_LOG` - Log level (default: `info`, options: `trace`, `debug`, `info`,
  `warn`, `error`)

### HTTP API Configuration

To enable the HTTP API in daemon mode, edit `~/.config/xrat/config.toml`:

```toml
[server]
enabled = true
host = "127.0.0.1"
port = 8080
```

For production with authentication:

```toml
[server]
enabled = true
host = "127.0.0.1"
port = 8080
key = { env = "XRAT_API_KEY" }
```

Then set the API key in the service file:

```ini
Environment=XRAT_API_KEY=your-secret-key-here
```

## System-Wide Installation

To install as a system service (requires root):

```bash
# Copy to system directory
sudo cp xrat-daemon.service /etc/systemd/system/

# Create dedicated user (optional but recommended)
sudo useradd -r -s /bin/false xrat
sudo mkdir -p /var/lib/xrat
sudo chown xrat:xrat /var/lib/xrat

# Edit service to use xrat user and system path
sudo nano /etc/systemd/system/xrat-daemon.service
# Add: User=xrat
# Change: Environment=XRAT_PATH=/var/lib/xrat

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable --now xrat-daemon.service

# Check status
sudo systemctl status xrat-daemon.service

# View logs
sudo journalctl -u xrat-daemon.service -f
```

## Troubleshooting

### Service won't start

```bash
# Check service status
systemctl --user status xrat-daemon.service

# View detailed logs
journalctl --user -u xrat-daemon.service -n 50

# Test manually
xrat daemon run-server
```

### HTTP API not responding

```bash
# Check if enabled in config
cat ~/.config/xrat/config.toml | grep -A 3 '\[server\]'

# Check logs for bind errors
journalctl --user -u xrat-daemon.service | grep -i 'http\|api\|bind'

# Test endpoint
curl http://127.0.0.1:8080/health
```

### Permission denied errors

```bash
# Check directory permissions
ls -la ~/.config/xrat/

# Fix ownership
chown -R $USER:$USER ~/.config/xrat/

# Check systemd sandboxing
systemctl --user show xrat-daemon.service | grep -i protect
```

## Stopping and Disabling

```bash
# Stop service
systemctl --user stop xrat-daemon.service

# Disable from startup
systemctl --user disable xrat-daemon.service

# Remove service file
rm ~/.config/systemd/user/xrat-daemon.service
systemctl --user daemon-reload
```

## Notes

- User services run only when the user is logged in (unless
  `loginctl enable-linger` is used)
- For always-on services, use system-wide installation or enable lingering:
  `loginctl enable-linger $USER`
- The daemon service includes auto-rotation and supervisor features
- The standalone API service is simpler but lacks daemon features
