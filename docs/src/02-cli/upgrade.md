# upgrade

Self-upgrade the running `xrat` binary, either by downloading the latest GitHub
release (default) or by building from a local source checkout.

```bash
xrat upgrade [OPTIONS]
```

The new binary is staged in the same directory as the current executable and
then atomically renamed over it, so an in-place upgrade is safe even while the
command is running.

## Flags

| Flag               | Description                                           | Default |
| ------------------ | ----------------------------------------------------- | ------- |
| `--source`         | Build and install from source instead of downloading  | off     |
| `--path <dir>`     | Source directory to build from when `--source` is set | `.`     |
| `--version <tag>`  | Download a specific release tag instead of the latest | latest  |
| `--force`          | Reinstall even when already on the requested version  | off     |
| `--timeout <secs>` | HTTP request timeout in seconds for release downloads | `120`   |

## Release upgrade (default)

```bash
xrat upgrade
```

1. Queries the latest GitHub release tag (or uses `--version`).
2. If the current binary already matches, prints `already using latest version`
   and exits without downloading. Use `--force` to reinstall anyway.
3. Downloads the matching `xrat-<version>-<arch>.tar.gz` archive with a progress
   bar, verifies it against `SHASUMS256.txt`, extracts the binary, and replaces
   the running executable.

Prebuilt archives are Linux-only (`x86_64` and `aarch64`, musl). On other
platforms or architectures, use `--source`.

```bash
xrat upgrade --version v0.2.1 --force
```

## Build from source

```bash
xrat upgrade --source            # builds from the current directory
xrat upgrade --source --path ~/code/xrat
```

Runs `cargo build --release` in the source directory, then installs the produced
`target/release/xrat` over the running binary. Requires `cargo` on `PATH` and a
`Cargo.toml` in the source directory.

## Notes

- Replacing a binary in a system directory (for example `/usr/local/bin`) may
  require elevated permissions; rerun with `sudo` if you hit a permission error.
- Only the binary is replaced. Man pages and shell completions are not updated;
  rerun `install.sh` if you want those refreshed too.

## Related

- [Installation Script](../01-getting-started/installation.md)
- [Build From Source](../01-getting-started/source-install.md)
