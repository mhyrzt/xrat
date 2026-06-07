# 08.8 Easy, P2: `install.sh` acceptance of macOS

**Difficulty:** Easy — 30 min.

**File:** `install.sh`

Currently refuses non-Linux at startup. Add macOS detection:

```bash
OS="$(uname -s)"
case "$OS" in
    Linux) ARCH="$(uname -m)" ;;
    Darwin)
        ARCH="$(uname -m)"
        [ "$ARCH" = "x86_64" ] && ARCH="x86_64" || ARCH="aarch64"
        ;;
    *) error "Unsupported OS: $OS"; exit 1 ;;
esac

# Skip systemd-related setup steps on macOS
if [ "$OS" = "Darwin" ]; then
    SKIP_DAEMON_INSTALL=1
    SKIP_LINGER=1
fi
```
