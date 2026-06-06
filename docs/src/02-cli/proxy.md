# proxy

Local proxy endpoints and host/session integration helpers.

```bash
xrat proxy <action> [flags]
```

> Automatic rotation scheduling moved to the [`rotate`](rotate.md) command. The
> old `xrat proxy start|status|stop` rotation commands have been removed.

## Actions

| Action      | Description                                      |
| ----------- | ------------------------------------------------ |
| `endpoints` | Show active local proxy endpoints                |
| `pac`       | Print or locate the Proxy Auto-Config (PAC) file |
| `shell`     | Proxy the current terminal session via env vars  |
| `desktop`   | Manage Linux desktop environment proxy settings  |

---

## proxy endpoints

Show active local proxy endpoints for quick copy/paste.

```bash
xrat proxy endpoints [--json]
```

Lists the active runtime inbounds (HTTP, SOCKS5, Shadowsocks) plus the PAC URL
when the API server is enabled. If an inbound binds `0.0.0.0`, the machine LAN
IP is shown for easy local-network use; otherwise the configured bind host is
shown.

Shadowsocks credentials are not persisted in runtime status, so the Shadowsocks
line shows only the endpoint with a `(credentials not shown)` note rather than a
leaky partial `ss://` URI.

### Flags

| Flag     | Description             |
| -------- | ----------------------- |
| `--json` | Print endpoints as JSON |

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
is disabled, a note explains how to enable it.

### proxy pac print

Generates the PAC file locally from the active runtime's HTTP/SOCKS inbounds and
prints it to stdout. The generated PAC:

- Routes plain hostnames, `*.local`, loopback, and private IP ranges `DIRECT`.
- Prefers SOCKS, then HTTP, then `DIRECT` for everything else.
- With no active runtime, routes everything `DIRECT`.

### PAC route

The PAC file is also served by the Axum API server at:

```
GET /proxy.pac
```

This route is **unauthenticated by default** and returns
`Content-Type: application/x-ns-proxy-autoconfig`. PAC consumers usually cannot
send auth headers, and the file exposes only non-secret local endpoint data
(Shadowsocks credentials are never included). Prefer a loopback server bind for
PAC use.

---

## proxy shell

Proxy only the current terminal session and its child processes, without
changing desktop or system proxy settings. xrat **prints** shell commands; it
never edits `.bashrc`, `.zshrc`, or fish config.

```bash
xrat proxy shell enable [--shell bash|zsh|fish]
xrat proxy shell disable [--shell bash|zsh|fish]
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
```

### Shell detection

The shell is detected from `$SHELL`, then the parent process name via
`/proc/$PPID/comm`, defaulting to bash. Override with `--shell bash|zsh|fish`.

---

## proxy desktop

Linux-only desktop proxy integration. This changes desktop environment proxy
settings, not every process on the system.

```bash
xrat proxy desktop enable [--desktop gnome|kde|xfce] [--pac]
xrat proxy desktop disable [--desktop gnome|kde|xfce]
xrat proxy desktop status [--desktop gnome|kde|xfce]
```

The desktop is auto-detected from `$XDG_CURRENT_DESKTOP` / `$DESKTOP_SESSION`;
override with `--desktop`. **GNOME** is supported first through `gsettings`:

- `enable` sets manual HTTP/HTTPS/SOCKS proxies from the active runtime, or with
  `--pac` switches to automatic mode using the PAC URL.
- `disable` resets the proxy mode to `none`.
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
