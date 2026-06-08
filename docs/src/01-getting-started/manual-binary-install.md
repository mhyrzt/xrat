# Manual Binary Install

Use this path when you want to inspect or place release files yourself instead
of running the installer script.

For the recommended path, see [Installation Script](installation.md). To compile
from this repository, see [Build From Source](source-install.md).

## Download

Go to the latest
[GitHub release](https://github.com/mhyrzt/xrat/releases/latest) and download
the archive for your architecture:

| File                                            | Architecture             |
| ----------------------------------------------- | ------------------------ |
| `xrat-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz`  | x86_64 (most PCs)        |
| `xrat-vX.Y.Z-aarch64-unknown-linux-musl.tar.gz` | ARM64 (Pi 4/5, Graviton) |

Download `SHASUMS256.txt` from the same release.

## Verify

Run the checksum verification from the directory containing the archive and
`SHASUMS256.txt`:

```bash
sha256sum -c SHASUMS256.txt --ignore-missing
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

## Install Shell Completions

The release archive includes generated completion files.

### Bash

```bash
mkdir -p ~/.local/share/bash-completion/completions
cp completions/xrat.bash ~/.local/share/bash-completion/completions/xrat
```

Reload by opening a new shell or sourcing your Bash startup file.

### Zsh

```bash
mkdir -p ~/.zfunc
cp completions/_xrat ~/.zfunc/_xrat
```

Add this to `~/.zshrc` if `~/.zfunc` is not already in `fpath`:

```zsh
fpath=("$HOME/.zfunc" $fpath)
autoload -Uz compinit
compinit
```

### Fish

```bash
mkdir -p ~/.config/fish/completions
cp completions/xrat.fish ~/.config/fish/completions/xrat.fish
```

## Install Man Pages

```bash
mkdir -p ~/.local/share/man/man1
cp man/man1/*.1 ~/.local/share/man/man1/
mandb ~/.local/share/man
```

If `mandb` is unavailable, the man pages are still copied; your system may index
them later.

## Install Desktop Entry

```bash
mkdir -p ~/.local/share/applications
mkdir -p ~/.local/share/icons/hicolor/48x48/apps
mkdir -p ~/.local/share/icons/hicolor/256x256/apps
cp desktop/xrat.desktop ~/.local/share/applications/
cp docs/src/media/icons/xrat-icon-48x48.png ~/.local/share/icons/hicolor/48x48/apps/xrat.png
cp docs/src/media/icons/xrat-icon-256x256.png ~/.local/share/icons/hicolor/256x256/apps/xrat.png
update-desktop-database ~/.local/share/applications/
```

The installer normally rewrites the desktop entry to use a detected terminal
emulator with xrat's window identity. For manual installs, you can either use the
static `desktop/xrat.desktop` entry, or generate a local wrapper such as:

```bash
cat > ~/.local/bin/xrat-desktop <<'EOF'
#!/usr/bin/env sh
exec kitty --class=xrat --title=XRAT "$HOME/.local/bin/xrat" tui "$@"
EOF
chmod +x ~/.local/bin/xrat-desktop
sed -i \
  -e 's|^Exec=.*|Exec='"$HOME"'/.local/bin/xrat-desktop|' \
  -e 's|^Terminal=.*|Terminal=false|' \
  ~/.local/share/applications/xrat.desktop
grep -q '^StartupWMClass=' ~/.local/share/applications/xrat.desktop \
  || printf '%s\n' 'StartupWMClass=xrat' >> ~/.local/share/applications/xrat.desktop
```

Use the terminal flag that matches your system: kitty `--class=xrat`, Alacritty
`--class xrat,xrat`, WezTerm `--class xrat`, foot `--app-id=xrat`, Konsole
`--desktopfile xrat`, GNOME Terminal `--class=xrat` on X11, or xterm
`-class xrat` on X11.

## First-Time Setup

```bash
xrat init
xrat daemon install --start
```

Then follow the [Quickstart](quickstart.md).
