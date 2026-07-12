# setup

Run post-install setup. `setup` performs the same work the install script used
to script in bash, but in the binary itself, so it works regardless of how xrat
was installed (release archive, `cargo install`, distro package, or manual
copy) and can be re-run any time to finish or repair an install.

```bash
xrat setup [OPTIONS]
```

## Flags

| Flag               | Description                                                          |
| ------------------ | ------------------------------------------------------------------- |
| `-y`, `--yes`      | Non-interactive; accept all recommended defaults                    |
| `--no-daemon`      | Do not install/start the background daemon                          |
| `--no-desktop`     | Skip the desktop launcher + icon install (Linux/XDG only)           |
| `--no-completions` | Skip shell completion install                                       |
| `--no-manpages`    | Skip man page install                                               |
| `--linger`         | Enable boot-before-login start (Linux; implies the daemon)          |
| `--check`          | Diagnose only: report what is/isn't set up, change nothing          |
| `--format <fmt>`   | Output format for `--check`: `table` (default) or `json`            |

`--check` cannot be combined with the mutating flags, and `--linger` cannot be
combined with `--no-daemon` (linger implies the daemon).

## Steps

`setup` runs these steps in order, each idempotent:

1. **Dependencies** — checks that `xray` is on `PATH` (required) and `sing-box`
   (optional). A missing `xray` aborts setup with an install hint.
2. **init** — creates the config directory, `config.toml`, database, and
   subdirectories (reuses [`init`](init.md); never overwrites a customized
   config).
3. **daemon** — installs and starts the background daemon (systemd user service
   on Linux, launchd agent on macOS, rc.d on BSD). Prompted unless `--yes`.
4. **linger** — *(Linux)* runs `loginctl enable-linger` so the daemon can start
   at boot before login. Forced with `--linger`; otherwise prompted (default no)
   in interactive runs, and skipped with `--yes`.
5. **completions** — generates and installs bash/zsh/fish completions into the
   standard XDG locations.
6. **man pages** — generates and installs man pages under
   `$XDG_DATA_HOME/man/man1`.
7. **desktop** — *(Linux/XDG)* installs a terminal-aware launcher, a `.desktop`
   entry, and hicolor icons.
8. **xratui** — installs an `xratui` shortcut script next to the `xrat` binary
   that execs `xrat tui`.
9. **PATH** — checks whether the binary's directory is on `PATH` and prints an
   export hint if not.

**Idempotent**: re-running reports each step as `already done` instead of
failing. Setup is recorded as a diagnostic event, so a run appears in
[`xrat logs`](logs.md).

## Example: guided setup

```bash
xrat setup -y
```

```
Environment
  os           linux
  arch         x86_64
  shell        fish
  terminal     kitty

Dependencies
  ✔ xray         /usr/local/bin/xray (v26.3.27)
  ✔ sing-box     /usr/bin/sing-box (v1.13.13)

Setup
  ✔ init         /home/user/.config/xrat
  ✔ daemon       installed and started
  ✔ completions  3 shells
  ✔ man pages    68 pages
  ✔ desktop      /home/user/.local/share/applications/xrat.desktop
  ✔ xratui       /home/user/.local/bin/xratui
  ✔ PATH         /home/user/.local/bin

OK Setup complete.
```

## Example: diagnose an install

```bash
xrat setup --check
```

```
  STEP         STATUS        DETAIL
✔ xray         done          /usr/local/bin/xray (v26.3.27)
✔ sing-box     done          /usr/bin/sing-box (v1.13.13)
✔ init         already done  /home/user/.config/xrat
✖ daemon       missing       background daemon not installed
✔ completions  already done  -
✔ man pages    already done  -
✖ desktop      missing       desktop launcher not installed
✔ xratui       already done  /home/user/.local/bin/xratui
✔ PATH         done          /home/user/.local/bin
```

`--check` exits non-zero when a required step (xray, init) is missing. Use
`--format json` for machine-readable output:

```bash
xrat setup --check --format json
```

## Relationship to install.sh

The [install script](../01-getting-started/installation.md) downloads, verifies,
and places the binary, then runs `xrat setup` (passing through `-y`,
`--no-desktop`, and `--linger`). If you installed xrat another way, run
`xrat setup` yourself to complete setup.

## Related

- [init](init.md) — just the config directory/database step
- [daemon install](daemon.md#daemon-install) — just the background service step
- [completions](completions.md) / [manpage](manpage.md) — generate scripts to stdout
- [Installation Script](../01-getting-started/installation.md)
