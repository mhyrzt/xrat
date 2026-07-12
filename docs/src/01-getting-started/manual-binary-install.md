# Manual Binary Install

Use this path when you want to inspect or place release files yourself instead
of running the installer script.

For the recommended path, see [Installation Script](installation.md). To compile
from this repository, see [Build From Source](source-install.md).

## Download

Go to the latest
[GitHub release](https://github.com/mhyrzt/xrat/releases/latest) and download
the archive for your platform:

| File                                            | Platform                 |
| ----------------------------------------------- | ------------------------ |
| `xrat-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz`  | Linux x86_64 (most PCs)  |
| `xrat-vX.Y.Z-aarch64-unknown-linux-musl.tar.gz` | Linux ARM64 (Pi, Graviton) |
| `xrat-vX.Y.Z-x86_64-apple-darwin.tar.gz`        | macOS Intel              |
| `xrat-vX.Y.Z-aarch64-apple-darwin.tar.gz`       | macOS Apple Silicon      |

Download `SHASUMS256.txt` from the same release.

## Verify

Run the checksum verification from the directory containing the archive and
`SHASUMS256.txt`. On Linux use `sha256sum`; on macOS use `shasum -a 256`:

```bash
sha256sum -c SHASUMS256.txt --ignore-missing   # Linux
shasum -a 256 -c SHASUMS256.txt --ignore-missing  # macOS
```

The command should report `OK` for the archive you downloaded.

## Install Binary

```bash
tar -xzf xrat-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz
mkdir -p ~/.local/bin
mv xrat ~/.local/bin/xrat
chmod +x ~/.local/bin/xrat
```

Ensure `~/.local/bin` is in `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Add that line to your shell startup file if needed.

## Run Setup

The release archive ships only the binary; man pages, shell completions, and the
desktop launcher are generated from the binary itself. Run [`xrat setup`](../02-cli/setup.md)
to install everything:

```bash
xrat setup
```

This is idempotent and re-runnable. It checks dependencies (`xray` required,
`sing-box` optional), runs `xrat init`, offers to install the background daemon,
and installs shell completions, man pages, an `xratui` shortcut, and (on
Linux/XDG) a terminal-aware desktop launcher with icons. Use `-y` to accept
defaults non-interactively, or flags like `--no-daemon` / `--no-desktop` to
skip individual steps.

To check what is and isn't configured without changing anything:

```bash
xrat setup --check
```

If you only want individual pieces: `xrat init` for the config directory and
database, `xrat daemon install --start` for the daemon, or
`xrat completions <shell>` / `xrat manpage --output <dir>` to print/generate
those assets yourself.

Then follow the [Quickstart](quickstart.md).
