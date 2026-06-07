# Build From Source

Use this path when you want to build from a checkout, test local changes, or
install a development build. The source workflow is Justfile-oriented; direct
Cargo commands are shown only where they help explain what each target does.

For release binaries, use [Installation Script](installation.md) or
[Manual Binary Install](manual-binary-install.md).

## Requirements

Install:

- `git`
- Rust via [rustup](https://rustup.rs/)
- `just`
- `xray` in `PATH`
- `sing-box` if you need sing-box preview or managed Hysteria2 runtime support

Install `just` with Cargo if your distribution does not package it:

```bash
cargo install just
```

Check the local task list:

```bash
just --list
```

## Clone

```bash
git clone https://github.com/mhyrzt/xrat.git
cd xrat
```

## Build

For a development build:

```bash
just build
```

For a release build using the locked dependency graph:

```bash
just release
```

The release binary is written to:

```text
target/release/xrat
```

Run a local command from the checkout:

```bash
just run status
```

## Install From Checkout

Install the current checkout to `~/.cargo/bin/xrat`:

```bash
just install
```

Replace an existing Cargo-installed binary:

```bash
just reinstall
```

Remove the Cargo-installed binary:

```bash
just uninstall
```

Ensure `~/.cargo/bin` is in `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Rustup usually adds this automatically. Add it to your shell startup file if
`xrat --version` cannot find the installed binary.

## Install Man Pages From Source

Generate and install man pages from the local command definitions:

```bash
just install-manpages
```

This writes pages to `~/.local/share/man/man1` and refreshes that man database
when `mandb` is available.

## Install Completions From Source

The `completions` target prints generated completions for the requested shell.
Redirect the output to the location your shell reads.

### Bash

```bash
mkdir -p ~/.local/share/bash-completion/completions
just completions bash > ~/.local/share/bash-completion/completions/xrat
```

Open a new shell or source your Bash startup file.

### Zsh

```bash
mkdir -p ~/.zfunc
just completions zsh > ~/.zfunc/_xrat
```

Add this to `~/.zshrc` if needed:

```zsh
fpath=("$HOME/.zfunc" $fpath)
autoload -Uz compinit
compinit
```

### Fish

```bash
mkdir -p ~/.config/fish/completions
just completions fish > ~/.config/fish/completions/xrat.fish
```

## First-Time Setup

Initialize the config directory and database:

```bash
xrat init
```

Install and start the systemd user daemon:

```bash
xrat daemon install --start
```

Then follow the [Quickstart](quickstart.md).

## Source-Tree Checks

Run the same commands as `.github/workflows/ci.yml`:

```bash
just ci
```

That expands to:

```bash
just fmt-rust-check
just lint
just test
```

For broader local formatting checks across Rust, Markdown, and SQL:

```bash
just fmt-check
```

Useful supporting targets:

| Target               | Purpose                                           |
| -------------------- | ------------------------------------------------- |
| `just check`         | Run `cargo check --locked`                        |
| `just fmt`           | Format Rust, Markdown, and SQL                    |
| `just fmt-check`     | Check Rust, Markdown, and SQL formatting          |
| `just docs`          | Serve the mdBook locally                          |
| `just clean`         | Remove Cargo build artifacts                      |
| `just postgres-up`   | Start the local PostgreSQL verification database  |
| `just test-postgres` | Run the PostgreSQL real-backend verification test |
| `just postgres-down` | Stop the local PostgreSQL verification database   |
