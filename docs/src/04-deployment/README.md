# Deployment

xrat can be deployed in various configurations, from single-user desktop setups
to multi-user server deployments with PostgreSQL.

## Deployment Options

| Option                                    | Description                   | Use Case                              |
| ----------------------------------------- | ----------------------------- | ------------------------------------- |
| [systemd](systemd.md)                     | Run as a systemd user service | Persistent daemon, auto-start on boot |
| [Database Backends](database-backends.md) | SQLite vs PostgreSQL          | Single-user vs multi-user deployments |

## Quick Deployment Checklist

1. **Build xrat**: `cargo build --release`
2. **Install binary**: Copy `target/release/xrat` to `/usr/local/bin/`
3. **Create config directory**: `mkdir -p ~/.config/xrat`
4. **Write config.toml**: Configure database, runtime, testing settings
5. **Import subscriptions**: `xrat import https://example.com/sub.txt`
6. **Test configs**: `xrat test --enabled-only`
7. **Start daemon**: `xrat daemon start` or use systemd
8. **Enable rotation** (optional): `xrat rotate start`
9. **Start HTTP API** (optional): `xrat serve` or enable in daemon

## Environment Variables

xrat respects these environment variables:

| Variable                    | Description                                       |
| --------------------------- | ------------------------------------------------- |
| `XRAT_PATH`                 | Config directory path (default: `~/.config/xrat`) |
| `RUST_LOG`                  | Log level (overrides `--verbose`/`--quiet`)       |
| `XRAT_API_KEY`              | HTTP API authentication key                       |
| `XRAT_SOCKS_PASSWORD`       | SOCKS inbound password                            |
| `XRAT_SHADOWSOCKS_PASSWORD` | Shadowsocks inbound password                      |
| `XRAT_POSTGRES_USER`        | PostgreSQL username                               |
| `XRAT_POSTGRES_PASSWORD`    | PostgreSQL password                               |

## Binary Dependencies

xrat requires external proxy binaries:

| Binary     | Required For                                                 | Installation                                                       |
| ---------- | ------------------------------------------------------------ | ------------------------------------------------------------------ |
| `xray`     | Managed runtime, most parse/test/generate flows              | [Xray-core releases](https://github.com/XTLS/Xray-core/releases)   |
| `v2ray`    | Alternative managed runtime binary                           | [V2Ray releases](https://github.com/v2fly/v2ray-core/releases)     |
| `sing-box` | sing-box JSON preview and managed Hysteria2 runtime sessions | [sing-box releases](https://github.com/SagerNet/sing-box/releases) |

Ensure binaries are in `PATH` or specify paths in `config.toml`:

```toml
[paths]
xray = "/usr/local/bin/xray"
v2ray = "/usr/local/bin/v2ray"
sing_box = "/usr/local/bin/sing-box"
```

Managed runtime process lifecycle uses Xray/V2Ray for their supported protocols.
Hysteria2 (`hy2`) configs are launched through sing-box automatically because
Xray/V2Ray cannot generate a compatible runtime config for them.

## Security Considerations

### File Permissions

Restrict access to config directory:

```bash
chmod 700 ~/.config/xrat
chmod 600 ~/.config/xrat/config.toml
chmod 600 ~/.config/xrat/db.sqlite
```

### Network Exposure

By default, xrat binds to:

- **SOCKS5**: `0.0.0.0:18200` (all interfaces)
- **HTTP API**: `127.0.0.1:18203` (localhost only)

To restrict SOCKS5 to localhost:

```toml
[runtime.socks]
host = "127.0.0.1"
```

To expose HTTP API externally (with authentication):

```toml
[server]
host = "0.0.0.0"
port = 18203
key = { env = "XRAT_API_KEY" }
```

### Secrets Management

Use environment variables for sensitive values:

```toml
[server]
key = { env = "XRAT_API_KEY" }

[runtime.socks]
auth = { enabled = true, username = "xrat", password = { env = "XRAT_SOCKS_PASSWORD" } }
```

Set in shell profile or systemd service:

```bash
export XRAT_API_KEY=$(openssl rand -hex 32)
export XRAT_SOCKS_PASSWORD=$(openssl rand -hex 16)
```

## Monitoring

### Health Checks

Use the HTTP API for monitoring:

```bash
curl http://localhost:8080/health
```

### Logs

View daemon logs:

```bash
journalctl --user -u xrat-daemon -f
```

Or with direct execution:

```bash
RUST_LOG=info xrat daemon start 2> daemon.log
```

### Process Monitoring

Check if daemon is running:

```bash
xrat daemon status
ps aux | grep xrat
```

## Backup and Recovery

### SQLite

Backup the database file:

```bash
cp ~/.config/xrat/db.sqlite ~/backup/db.sqlite.$(date +%Y%m%d)
```

### PostgreSQL

Use `pg_dump`:

```bash
pg_dump xrat > ~/backup/xrat.$(date +%Y%m%d).sql
```

### Config Files

Backup config directory:

```bash
tar czf ~/backup/xrat-config.$(date +%Y%m%d).tar.gz ~/.config/xrat/
```

## Troubleshooting

### Daemon Won't Start

**Check**:

- Is a daemon already running? `xrat daemon status`
- Check logs: `RUST_LOG=debug xrat daemon start`
- Verify socket directory is writable

### Connection Failed

**Check**:

- Is Xray binary available? `which xray`
- Test config manually: `xrat test <id>`
- Check runtime logs: `~/.config/xrat/runtime/session-*.err.log`

### Database Locked (SQLite)

**Symptom**: "database is locked" errors

**Fix**:

- Only one process can write to SQLite at a time
- Use PostgreSQL for multi-user deployments
- Increase busy timeout in config.toml (if supported)

## Related

- [systemd](systemd.md) — systemd service examples
- [Database Backends](database-backends.md) — SQLite vs PostgreSQL
- [Configuration](../01-getting-started/configuration.md) — config.toml
  reference
