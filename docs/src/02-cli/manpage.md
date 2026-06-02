# manpage

Generate roff-format man pages for xrat and all subcommands.

```bash
xrat manpage [--output <dir>]
```

This is a hidden command (not shown in `--help`) intended for use during release
packaging and local installation.

## Flags

| Flag             | Description                             | Default |
| ---------------- | --------------------------------------- | ------- |
| `--output <dir>` | Directory to write generated `.1` files | `.`     |

## Behavior

Generates one man page per visible command and subcommand:

- `xrat.1` — root command with global flags
- `xrat-init.1`, `xrat-import.1`, `xrat-daemon.1`, ... — top-level subcommands
- `xrat-daemon-install.1`, `xrat-daemon-stop.1`, ... — nested subcommands

Hidden commands (e.g., `daemon run-server`) are excluded.

Output format is roff/troff compatible with `man(1)`.

## Example

```bash
xrat manpage --output /tmp/man
```

```
/tmp/man/xrat.1
/tmp/man/xrat-init.1
/tmp/man/xrat-import.1
/tmp/man/xrat-daemon.1
/tmp/man/xrat-daemon-install.1
...
```

## Installing locally

```bash
mkdir -p ~/.local/share/man/man1
xrat manpage --output ~/.local/share/man/man1
mandb ~/.local/share/man   # update index (may require once)
man xrat
man xrat-daemon-install
```

Or system-wide:

```bash
sudo xrat manpage --output /usr/local/share/man/man1
sudo mandb
```

## Release packaging

CI generates man pages during the release workflow and includes them in release
archives under `man/`:

```bash
xrat manpage --output dist/man/man1/
```

## Related

- [Quickstart](../01-getting-started/quickstart.md)
- [`init`](init.md)
- [`daemon`](daemon.md)
