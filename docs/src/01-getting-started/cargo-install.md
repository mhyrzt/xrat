# Cargo Install

Install xrat from [crates.io](https://crates.io/crates/xrat) with Cargo. Use this
path if you already have a Rust toolchain and want the published crate without
the install script.

For release binaries, see [Installation Script](installation.md) or
[Manual Binary Install](manual-binary-install.md). To build from a checkout, see
[Build From Source](source-install.md).

## Requirements

- Rust toolchain via [rustup](https://rustup.rs/) (`cargo` in `PATH`)
- `xray` in `PATH` (required); `sing-box` if you need sing-box preview or managed
  Hysteria2 runtime support

## Install

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
cargo install xrat        # reinstall/upgrade to the latest published version
cargo uninstall xrat      # remove the Cargo-installed binary
```

`xrat upgrade` self-upgrades a release-archive install; for a Cargo-installed
binary, prefer `cargo install xrat` to update.
