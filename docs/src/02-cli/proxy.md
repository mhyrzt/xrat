# proxy

Local proxy endpoints and host/session integration helpers.

```bash
xrat proxy <action> [flags]
```

> Automatic rotation scheduling moved to the [`rotate`](rotate.md) command. The
> old `xrat proxy start|status|stop` rotation commands have been removed.

## Actions

| Action    | Description                                      |
| --------- | ------------------------------------------------ |
| `info`    | Show active local proxy endpoints                |
| `pac`     | Print or locate the Proxy Auto-Config (PAC) file |
| `shell`   | Proxy the current terminal session via env vars  |
| `desktop` | Manage Linux desktop environment proxy settings  |

---

## proxy info

Show active local proxy endpoints for the current runtime.

```bash
xrat proxy info [--json]
```

Lists the active runtime inbounds (HTTP, SOCKS5, Shadowsocks) plus the PAC URL
when `[server].enabled = true` and `[server].pac_enabled = true`. If an inbound
binds `0.0.0.0`, the machine LAN IP is shown for easy local-network use;
otherwise the configured bind host is shown.

Shadowsocks credentials are not persisted in runtime status, so the Shadowsocks
line shows only the endpoint with a `(credentials not shown)` note rather than a
leaky partial `ss://` URI.

`xrat proxy show` and `xrat proxy endpoints` are accepted as aliases.

### Flags

| Flag     | Description                     |
| -------- | ------------------------------- |
| `--json` | Print proxy information as JSON |

---

## proxy pac

Work with a Proxy Auto-Config (PAC) file generated from the active runtime
endpoints, so browsers and desktop environments can use per-destination routing
instead of a blunt global proxy.

```bash
xrat proxy pac url     # print the PAC URL served by the API server
xrat proxy pac print   # print the generated PAC file for the active runtime
```

### proxy pac url

Prints the URL the API server serves the PAC file at, for example
`http://127.0.0.1:8787/proxy.pac`. A wildcard bind host (`0.0.0.0`) is shown as
`127.0.0.1`, since PAC consumers should fetch over loopback. If the API server
or PAC route is disabled, a note explains which `[server]` setting to enable.

### proxy pac print

Generates the PAC file locally from the active runtime's HTTP/SOCKS inbounds and
prints it to stdout. The generated PAC:

- Routes plain hostnames, `*.local`, loopback, and private IP ranges `DIRECT`.
- Applies curated `[routing.direct]` and `[routing.block]` `domain` entries and
  IPv4 CIDRs from `ip` lists in that order.
- Prefers SOCKS, then HTTP, for everything else; no `DIRECT` fallback is added
  while a proxy is active.
- With no active runtime, routes everything `DIRECT`.
- Rewrites wildcard inbound hosts like `0.0.0.0` to `127.0.0.1` because PAC
  clients need a concrete proxy destination.
- Resolves hostnames before private IPv4 CIDR checks and skips those checks when
  DNS resolution fails.

PAC generation does not inline `geosite` or `geoip` lists; those stay in the
proxy engine config.

### PAC route

The PAC file is also served by the Axum API server at:

```
GET /proxy.pac
```

This route is **unauthenticated by default** and returns
`Content-Type: application/x-ns-proxy-autoconfig`. PAC consumers usually cannot
send auth headers, and the file exposes only non-secret local endpoint data
(Shadowsocks credentials are never included). Prefer a loopback server bind for
PAC use. Set `[server].pac_enabled = false` to disable this route. Requests are
accepted only when the HTTP `Host` header matches `[server].pac_allowed_hosts`,
which defaults to `localhost`, `127.0.0.1`, and `::1`.

---

## proxy shell

Proxy only the current terminal session and its child processes, without
changing desktop or system proxy settings. xrat **prints** shell commands; it
never edits `.bashrc`, `.zshrc`, or fish config.

```bash
xrat proxy shell enable [--shell bash|zsh|fish]
xrat proxy shell disable [--shell bash|zsh|fish]
xrat proxy shell toggle [--shell bash|zsh|fish]
xrat proxy shell status
```

`enable` sets `http_proxy`/`https_proxy` (prefer the HTTP inbound, falling back
to SOCKS) and `all_proxy` (prefer SOCKS, falling back to HTTP), plus their
uppercase variants. It errors if no usable inbound is active. `disable` unsets
those variables. `status` inspects the environment inherited by `xrat` and
reports whether the current shell points at active xrat endpoints.

### Usage

bash/zsh — a child process cannot mutate its parent shell's environment, so eval
the output:

```sh
eval "$(xrat proxy shell enable)"
eval "$(xrat proxy shell disable)"
```

fish — source from a pipe:

```fish
xrat proxy shell enable | source
xrat proxy shell disable | source
```

Optional convenience aliases (xrat does not create these for you):

```sh
alias xrat-proxy-on='eval "$(xrat proxy shell enable)"'
alias xrat-proxy-off='eval "$(xrat proxy shell disable)"'
alias xrat-proxy-toggle='eval "$(xrat proxy shell toggle)"'
```

### Shell detection

The shell is detected from `$SHELL`, then the parent process name, defaulting to
bash. Override with `--shell bash|zsh|fish`.

### Shell toggle

`toggle` prints shell commands. Use `eval "$(xrat proxy shell toggle)"` for
bash/zsh or pipe to `source` in fish.

When enabling, it captures the current proxy variables in temporary
`XRAT_PROXY_OLD_*` / `XRAT_PROXY_HAD_*` variables before exporting the active
xrat endpoints. When the shell already points at active xrat endpoints, the next
toggle restores the captured values or unsets variables that were absent.

---

## proxy desktop

Linux-only desktop proxy integration. This changes desktop environment proxy
settings, not every process on the system.

```bash
xrat proxy desktop enable [--desktop gnome|kde|xfce] [--pac]
xrat proxy desktop disable [--desktop gnome|kde|xfce]
xrat proxy desktop toggle [--desktop gnome|kde|xfce] [--pac]
xrat proxy desktop status [--desktop gnome|kde|xfce]
```

The desktop is auto-detected from `$XDG_CURRENT_DESKTOP` / `$DESKTOP_SESSION`;
override with `--desktop`. **GNOME** is supported first through `gsettings`:

- `enable` sets manual HTTP/HTTPS/SOCKS proxies from the active runtime by
  default and does not require PAC. With `--pac`, it switches to automatic mode
  using the PAC URL; PAC mode requires both `[server].enabled = true` and
  `[server].pac_enabled = true`.
- `disable` resets the proxy mode to `none`.
- `toggle` enables manual HTTP/HTTPS/SOCKS settings when the current mode is
  `none`; with `--pac`, it uses PAC only while turning proxy on. If the current
  mode is not `none`, it disables without requiring PAC.
- `status` prints the current proxy mode.

KDE and XFCE are not supported yet and return a clear error suggesting
`xrat proxy shell enable` for terminal-only proxying. On non-Linux platforms the
command errors. `desktop` is used rather than `system` because Linux has no
single universal system proxy authority.

## Related

- [`rotate`](rotate.md) — automatic rotation scheduling
- [`daemon`](daemon.md) — daemon must be running for runtime operations
- [`connect`](runtime.md#connect) — start one proxy session through the daemon
- [`serve`](serve.md) — run the API server that hosts `/proxy.pac`
