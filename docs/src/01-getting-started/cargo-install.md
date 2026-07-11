# Cargo Install

Install xrat with Cargo. Use this path if you already have a Rust toolchain
and want `cargo`-managed installs without the install script.

For release binaries, see [Installation Script](installation.md) or
[Manual Binary Install](manual-binary-install.md). To build from a checkout, see
[Build From Source](source-install.md).

## Requirements

- Rust toolchain via [rustup](https://rustup.rs/) (`cargo` in `PATH`)
- `xray` in `PATH` (required); `sing-box` if you need sing-box preview or managed
  Hysteria2 runtime support

## Install with cargo-binstall (recommended)

[cargo-binstall](https://github.com/cargo-bins/cargo-binstall) downloads the
matching prebuilt release binary instead of compiling from source, so it's as
fast as the install script but stays inside your Cargo toolchain:

```bash
cargo binstall xrat
```

Install `cargo-binstall` itself first if you don't have it:

```bash
cargo install cargo-binstall
```

## Install from crates.io (builds from source)

```bash
cargo install xrat
```

Cargo places the binary in `~/.cargo/bin/xrat`. Ensure that directory is in
`PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add that line to your shell startup file if `xrat --version` cannot find it.

## Run Setup

`cargo install` only places the binary. Run [`xrat setup`](../02-cli/setup.md) to
complete setup — it checks dependencies, runs `xrat init`, offers to install the
background daemon, and installs shell completions, man pages, and (on Linux/XDG)
a desktop launcher with icons:

```bash
xrat setup
```

Setup is idempotent and re-runnable. Use `-y` to accept defaults
non-interactively, `--no-daemon` / `--no-desktop` to skip steps, or
`xrat setup --check` to report what is and isn't configured without changing
anything.

Then follow the [Quickstart](quickstart.md).

## Update and Uninstall

```bash
cargo binstall xrat       # reinstall/upgrade via prebuilt binary
cargo install xrat        # reinstall/upgrade by building from source
cargo uninstall xrat      # remove the Cargo-installed binary
```

`xrat upgrade` self-upgrades a release-archive install; for a Cargo-installed
binary, prefer `cargo binstall xrat` or `cargo install xrat` to update.
