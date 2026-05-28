# Quickstart

This guide walks through the core xrat workflow: import a subscription, test
configs, and start a local proxy.

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
xrat list configs --subscription 1
```

## 3. Test Connectivity

Test a single config by ID:

```bash
xrat test 1
```

Bulk-test all enabled configs:

```bash
xrat test --enabled-only --concurrency 4
```

Skip specific stages:

```bash
xrat test 1 --skip-icmp --skip-download
```

## 4. Start a Proxy

Connect using a tested config:

```bash
xrat connect 1
```

This starts the Xray (or V2Ray) process with a generated runtime config. By
default, it exposes:

- **SOCKS5** on `0.0.0.0:1080`
- **HTTP** on `0.0.0.0:8080` (if enabled in config.toml)

## 5. Check Status

```bash
xrat status
```

## 6. Disconnect

```bash
xrat disconnect
```

## Using the Daemon

For persistent operation with auto-rotation:

```bash
xrat daemon start
xrat proxy start
xrat proxy rotate
xrat proxy status
xrat daemon stop
```

See [daemon](../02-cli/daemon.md) and [proxy](../02-cli/proxy.md) for details.
