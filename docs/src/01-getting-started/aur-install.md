# Arch Linux (AUR)

xrat is available in the Arch User Repository (AUR) as both a prebuilt binary
package and a source package.

For other install paths, see [Installation Script](installation.md),
[Cargo Install](cargo-install.md), [Docker Install](docker-install.md), or
[Manual Binary Install](manual-binary-install.md).

## Packages

| Package | Description | Recommended |
| ------- | ----------- | :---------: |
| [`xrat-bin`](https://aur.archlinux.org/packages/xrat-bin) | Downloads and installs prebuilt release binaries directly from GitHub releases | Yes |
| [`xrat`](https://aur.archlinux.org/packages/xrat) | Builds `xrat` from source using `cargo` | - |

## Installation

### Using an AUR Helper

With `yay`:

```bash
# Prebuilt binary (recommended)
yay -S xrat-bin

# Or build from source:
yay -S xrat
```

With `paru`:

```bash
# Prebuilt binary (recommended)
paru -S xrat-bin

# Or build from source:
paru -S xrat
```

### Manual Installation (makepkg)

You can clone the repository and build manually without an AUR helper:

```bash
# For xrat-bin
git clone https://aur.archlinux.org/xrat-bin.git
cd xrat-bin
makepkg -si

# Or for xrat (from source)
git clone https://aur.archlinux.org/xrat.git
cd xrat
makepkg -si
```

## First-Time Setup

AUR packages install the `xrat` binary to `/usr/bin/xrat` alongside system
assets (desktop launcher, completions, man pages, service templates).

To initialize the configuration directory, database, and managed proxy cores,
run setup:

```bash
xrat setup
```

Setup is idempotent and re-runnable. Use `-y` to accept defaults non-interactively,
or `xrat setup --check` to verify your environment without making changes.

Then follow the [Quickstart](quickstart.md) to import configs and connect.

## Update and Uninstall

### Update

Using your AUR helper:

```bash
yay -Syu
# or
paru -Syu
```

### Uninstall

Remove the package and unused dependencies:

```bash
sudo pacman -Rns xrat-bin
# or
sudo pacman -Rns xrat
```
