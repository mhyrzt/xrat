<p align="center">
  <img src="docs/src/media/icons/xrat-icon-1024x1024.png" alt="xrat" width="256">
</p>

<h1 align="center">XRAT - Xray-core and sing-box Proxy Manager</h1>

<p align="center">
  <em>A Rust CLI and TUI proxy configuration manager for <a href="https://github.com/xtls/xray-core">XTLS/Xray-core</a>, <a href="https://github.com/v2fly/v2ray-core">V2Ray-core</a>, and <a href="https://github.com/sagernet/sing-box">SagerNet/sing-box</a></em>
</p>

<p align="center">
  <img alt="Status" src="https://img.shields.io/badge/status-under%20development-orange">
  <img alt="Rust" src="https://img.shields.io/badge/rust-stable-blue">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green">
  <a href="https://mhyrzt.github.io/xrat"><img src="https://img.shields.io/badge/docs-mhyrzt.github.io%2Fxrat-blue" alt="Documentation"></a>
  <a href="https://crates.io/crates/xrat"><img src="https://img.shields.io/crates/v/xrat" alt="crates.io"></a>
</p>

Import your subscriptions, find a working proxy, and keep it running from the
terminal. XRAT brings testing, connection management, and troubleshooting into
one Rust CLI and TUI, with saved results you can return to later.

- **Choose with measurements.** Test a whole subscription concurrently, sort by
  real HTTP latency through each proxy, and see which configs failed and why.
- **Keep a local proxy running.** Connect once, let the daemon supervise the
  process, and optionally rotate on a schedule or after health failures.
- **Use it where you work.** Browse the TUI, proxy a shell session, configure
  desktop proxy settings, or export results for scripts.

<p align="center">
  <img src="docs/src/media/gif/tui.gif" alt="XRAT terminal UI showing proxy testing progress, config details, logs, and runtime status">
</p>

The TUI keeps configs, test progress, runtime status, and logs on one dashboard.
Search and filter nodes, connect with `Enter`, share a config as a QR code with
`y`, or edit settings with `,`. Run `xrat tui` (or `xratui` after setup).

[Quickstart](#quickstart) · [CLI workflows](#cli-workflows) ·
[Documentation](https://mhyrzt.github.io/xrat)

## Installation

Install script (Linux/macOS):

```bash
curl -fsSL https://raw.githubusercontent.com/mhyrzt/xrat/master/install.sh | bash
```

Launch the TUI with `xratui` (or `xrat tui`).

The installer downloads `xrat`, then runs guided `xrat setup` to initialize
storage, install proxy cores, and configure the daemon, shell completions, man
pages, and the `xratui` shortcut. Desktop launcher integration is available on
Linux.

Or install with Cargo:

```bash
cargo binstall xrat   # prebuilt binary; requires cargo-binstall
# Or build from source:
cargo install xrat

xrat setup
```

Launch the TUI with `xratui` (or `xrat tui`).

XRAT manages external proxy engines: Xray is the default, V2Ray is an
alternative, and sing-box is available for every supported protocol (VLESS,
VMess, Shadowsocks, Trojan, HTTP, SOCKS5, and Hysteria2). Setup can install
managed copies of the cores; they do not need to be installed beforehand.
Managed sing-box use requires a sing-box `>=1.13.0` binary; newer versions are
accepted, with conformance fixtures covering `>=1.13.0, <1.15.0`.

See the [installation guide](docs/src/01-getting-started/installation.md) for
manual downloads, Docker, and building from a checkout.

### Upgrade XRAT

Already installed? Update XRAT with:

```bash
xrat upgrade
```

### Install proxy cores

Setup can install these for you, or you can install managed copies individually:

```bash
xrat install xray
xrat install sing-box
xrat install v2ray
```

Choose the cores you need. Each command downloads the latest stable release from
its official repository and configures XRAT to use it. See
[core installation](docs/src/02-cli/install.md) for details.

## Quickstart

Import your subscription, test its configs, and pick one to connect:

```bash
xrat import 'https://your-provider.example/subscription'
xrat test
xrat connect a1b2
```

Replace the URL with yours and `a1b2` with a config ref from the test results.
You can also import a local file with `xrat import ./links.txt`. If the daemon
is not already running after setup, run `xrat daemon start` first.

Your local SOCKS proxy is now available on port `18200`. Use it from your
terminal with the [shell helper below](#use-your-proxy-in-the-terminal), or open
`xratui` to manage connections interactively.

```bash
xrat status       # check the connection
xrat disconnect   # stop it when finished
```

## CLI workflows

A few everyday workflows below. Sample output uses fictional names, refs, and
measurements; the columns shown depend on your settings and results.

### Bring your existing configs

Import a subscription from your provider:

```bash
xrat import 'https://your-provider.example/subscription'
```

```text
OK Imported 12 parsed nodes.
database         /home/user/.config/xrat/db.sqlite
config           /home/user/.config/xrat/config.toml
subscription     f00d1234abcd
removed configs  0
total configs    12
```

Have a file instead? Plain link lists, base64 subscriptions, SIP008, and Xray
JSON are detected automatically:

```bash
xrat import ./links.txt
xrat import ./subscription.b64
xrat import ./sip008.json
xrat import ./xray.json
```

Add individual configs with `xrat add`, followed by a supported share link.
Examples for each type (replace the sample addresses and credentials with
yours):

```bash
# VLESS
xrat add 'vless://00000000-0000-0000-0000-000000000001@proxy.example:443?security=tls#VLESS'

# VMess
xrat add 'vmess://00000000-0000-0000-0000-000000000001@proxy.example:443?security=tls&encryption=auto#VMess'

# Trojan
xrat add 'trojan://password@proxy.example:443#Trojan'

# Shadowsocks
xrat add 'ss://YWVzLTI1Ni1nY206c2VjcmV0@proxy.example:8388#Shadowsocks'

# HTTP / HTTPS
xrat add 'http://user:password@proxy.example:8080#HTTP'
xrat add 'https://user:password@proxy.example:443#HTTPS'

# SOCKS5
xrat add 'socks5://user:password@proxy.example:1080#SOCKS5'

# Hysteria2
xrat add 'hy2://password@proxy.example:443#Hysteria2'
```

Example output after adding one config:

```text
OK Added 1 config.
database       /home/user/.config/xrat/db.sqlite
config         /home/user/.config/xrat/config.toml
source         raw_text
total configs  13
```

Each command saves a config without creating a subscription.

XRAT deduplicates configs as it imports them. Once imported, they are available
in both `xrat list configs` and the TUI. See the
[import guide](docs/src/02-cli/import.md) for more formats and details.

### Browse what you imported

List your subscriptions and their last update time:

```bash
xrat list subscriptions
```

```text
REF       KIND  CONFIGS  NAME      SOURCE                                     UPDATED AT
f00d1234  url        12  My nodes  https://provider.example/subscription      2026-09-12 10:30:00
```

List individual configs, including saved test results and the active connection:

```bash
xrat list configs
```

Example rows with GeoIP data available:

```text
REF       SUB       STATUS          PROTO  ADDRESS         PORT  ICMP   TCP   REAL  COUNTRY  CITY         ASN              NAME
a1b2c3d4  f00d1234  enabled         vless  edge-a.example   443  32ms  38ms  182ms  DE       DE/Frankfurt AS64500 Example  Frankfurt
b2c3d4e5  f00d1234  enabled,active  vless  edge-b.example   443  24ms  29ms  146ms  NL       NL/Amsterdam AS64501 Example  Amsterdam
c3d4e5f6  f00d1234  enabled         vless  edge-c.example   443     -     -      -  FR       FR/Paris     AS64502 Example  Paris
```

Use the `REF` to test or connect to a node, and the `SUB` ref to update its
subscription. GeoIP describes the dial endpoint; it is not a verified exit
location.

### Find a working proxy

Test your stored configs in one go:

```bash
xrat test
```

```text
REF       STATUS  ICMP   REAL  PROTO   ADDRESS         PORT  NAME
a1b2c3d4  ✓ ok    24ms  142ms  vless   edge-a.example   443  Primary
b2c3d4e5  ✓ ok    48ms  218ms  trojan  edge-b.example   443  Backup
c3d4e5f6  ✗ fail     -      -  vmess   edge-c.example  8443  Old node

Failures:
  • tcp: connection refused
    c3d4e5f6
```

`REAL` is HTTP latency through the proxy. Results are saved, so you can come
back to them with `xrat list configs`. To check just one node, use
`xrat test a1b2`.

There is more when you need it: concurrent testing, sorting, continuous ping,
throughput measurements, and CSV/JSON exports. See the
[testing guide](docs/src/02-cli/test.md).

### 🔄 Refresh your subscriptions

Fetch changes from the subscriptions you have already imported:

```bash
xrat update
```

```text
INFO All subscriptions updated!
Subscription update
attempted         2
succeeded         2
failed            0
imported configs  36
removed configs   4
```

To update just one subscription, find its ref with `xrat list subscriptions`,
then run `xrat update f00d`. Run `xrat test` again to check the refreshed nodes.

### Switch to another proxy

Let XRAT test candidates and switch the running connection:

```bash
xrat rotate now
```

```text
  rotation candidate tests 12/12 [================================] 12/12
INFO Rotation candidate testing finished: 12/12.
INFO [test:test_run] Tested 12 config(s): 4 passed, 8 failed
OK Proxy rotation completed.
replaced     yes
old session  1042
new config   b2c3d4e5f607
new session  1043
new pid      28416
```

Want it to happen automatically? `xrat rotate enable` turns on scheduled
rotation using your settings. `xrat rotate disable` turns it off while keeping
the current connection running. See the
[rotation guide](docs/src/02-cli/rotate.md) for scheduling and health-based
switching.

### Use your proxy in the terminal

With a config connected, run:

```sh
xrat proxy shell enable
```

XRAT detects your shell, chooses an available local proxy, and prints how to
apply it. In **fish**:

```fish
xrat proxy shell enable | source
curl https://example.com
xrat proxy shell disable | source
```

In **Bash/Zsh**:

```bash
eval "$(xrat proxy shell enable)"
curl https://example.com
eval "$(xrat proxy shell disable)"
```

This works with proxy-aware tools launched from that terminal. For desktop apps,
`xrat proxy desktop enable` supports GNOME on Linux and network services on
macOS. See [proxy integration](docs/src/02-cli/proxy.md) for details.

### See what happened

Connection dropped or rotation picked another node? Check the combined app,
daemon, and proxy-engine logs:

```bash
xrat logs
```

The same logs are visible in the TUI alongside your runtime status. The
[logs guide](docs/src/02-cli/logs.md) covers live following and filtering.

## 🧰 A few more useful commands

| Command                   | What it does                                |
| ------------------------- | ------------------------------------------- |
| `xrat list configs`       | Browse saved nodes and their latest results |
| `xrat list subscriptions` | Find your imported subscriptions            |
| `xrat show config a1b2`   | Inspect a node's details                    |
| `xrat disable a1b2`       | Keep a node but leave it out of rotation    |
| `xrat enable a1b2`        | Make it eligible again                      |
| `xrat daemon status`      | Check the background daemon                 |
| `xrat upgrade`            | Update XRAT itself                          |

Use refs from your own lists in place of the examples. For the full command set,
see the [CLI reference](docs/src/02-cli/README.md).

## More to explore

- **Protocol support:** VLESS, VMess, Trojan, Shadowsocks, HTTP/HTTPS, and
  SOCKS5 and Hysteria2 through Xray; Hysteria2 also works through sing-box. See
  [protocol details](docs/src/05-reference/protocols.md).
- **CDN scanning:** compare candidate edge IPs for TCP reachability and latency,
  then revisit saved results. See [scanning](docs/src/02-cli/scan.md).
- **Sharing and automation:** QR codes in the TUI, structured CLI exports, and
  an [HTTP API](docs/src/02-cli/serve.md) for configs, results, and
  subscriptions.
- **Routing and storage:** direct/proxy/block rules, configurable DNS and local
  inbounds, SQLite by default, and optional PostgreSQL. See
  [configuration](docs/src/05-reference/config-file.md).

## Acknowledgments

Some functionalities in XRAT have been inspired by
[xray-knife](https://github.com/lilendian0x00/xray-knife).
