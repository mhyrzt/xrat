# Quickstart

This guide walks through the core xrat workflow: import a subscription, test
configs, and start a local proxy.

## 0. Initialize

Run once after installing to create the config directory, default config file,
and database:

```bash
xrat init
```

See [`init`](../02-cli/init.md) for details on what gets created and how to use
a custom path via `XRAT_PATH`.

## 1. Import a Subscription

Import from a URL:

```bash
xrat import https://example.com/subscription
```

Import from a local file:

```bash
xrat import ./subscription.txt
```

Import raw subscription text directly:

```bash
xrat import "vless://uuid@example.com:443?type=ws&security=tls#MyNode"
```

xrat automatically detects the input format: subscription URL, local file, raw
base64-encoded subscription, plain link list, SIP008 JSON, or Xray JSON.

## 2. List Imported Configs

```bash
xrat list configs
```

Filter by subscription source:

```bash
xrat list configs --subscription f00d
```

Use `enable` and `disable` to control whether a config appears in enabled-only
workflows:

```bash
xrat disable a1b2
xrat enable a1b2
```

## 3. Test Connectivity

Test a single config by ref:

```bash
xrat test a1b2
```

Bulk-test all enabled configs:

```bash
xrat test --enabled-only --concurrency 4
```

Skip specific stages:

```bash
xrat test a1b2 --skip-icmp --skip-download
```

## 4. Start a Proxy

Start the daemon first:

```bash
xrat daemon start
```

Connect using a tested config:

```bash
xrat connect a1b2
```

The command sends a daemon IPC request. The daemon starts the Xray (or V2Ray)
process with a generated runtime config. By default, it exposes:

- **SOCKS5** on `0.0.0.0:18200`
- **HTTP** on `0.0.0.0:18201` (if enabled in config.toml)

## 5. Check Status

```bash
xrat status
```

## 6. Disconnect

```bash
xrat disconnect
```

## Interactive TUI

For an interactive view over configs, sources, tests, runtime status, and
diagnostics:

```bash
xrat tui
```

## Using the Daemon

For persistent operation with auto-rotation:

```bash
xrat daemon start
xrat rotate enable
xrat rotate now
xrat rotate status
xrat daemon stop
```

`xrat connect <id>` starts one managed runtime session immediately through the
daemon. `xrat rotate enable` enables daemon-driven auto-rotation.

See [daemon](../02-cli/daemon.md) and [proxy](../02-cli/proxy.md) for details.
