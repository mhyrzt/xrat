# 10.10 Easy, P2: Portable Unix installer paths and checksum tools

**Difficulty:** Easy — 1 hour.

**File:** `install.sh`

Once macOS is accepted by the installer, keep the install flow Unix-portable
instead of only Linux-shaped.

Current Linux assumptions to split by OS:

- `sha256sum` is required for release verification, but macOS ships `shasum`
  instead. Add a helper that runs `sha256sum -c` on Linux and `shasum -a 256 -c`
  on macOS/BSD where available.
- systemd setup prompts (`xrat daemon install --start`) and `loginctl` lingering
  should only appear on Linux/systemd. macOS should either skip daemon setup or
  offer launchd once item 04 is complete.
- Linux desktop launcher assets (`.desktop`, hicolor icons, `update-desktop-database`)
  should be skipped by default on macOS. Keep them Linux/XDG-only.
- OS/arch detection should return release target triples, not only architecture:
  - Linux x86_64/aarch64 → existing musl triples
  - macOS x86_64/aarch64 → darwin triples from item 06

Suggested shape:

```bash
detect_target() {
    case "$(uname -s):$(uname -m)" in
        Linux:x86_64) echo "x86_64-unknown-linux-musl" ;;
        Linux:aarch64) echo "aarch64-unknown-linux-musl" ;;
        Darwin:x86_64) echo "x86_64-apple-darwin" ;;
        Darwin:arm64|Darwin:aarch64) echo "aarch64-apple-darwin" ;;
        *) error "Unsupported platform: $(uname -s) $(uname -m)"; exit 1 ;;
    esac
}
```

**Verification:** run installer dry/manual checks on Linux and macOS, including
checksum verification and a setup flow where daemon/desktop prompts are skipped
or OS-appropriate.
