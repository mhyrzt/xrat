# 06.6 Medium, P2: Release upgrades for macOS

**Difficulty:** Medium — half day + CI changes.

**Files:** `src/app/commands/upgrade/release.rs`,
`.github/workflows/release.yml`

**Code change:** Add macOS arch detection to `detect_arch()`:

```rust
fn detect_arch() -> crate::app::Result<&'static str> {
    #[cfg(target_os = "macos")] {
        return match std::env::consts::ARCH {
            "x86_64" => Ok("x86_64-apple-darwin"),
            "aarch64" => Ok("aarch64-apple-darwin"),
            other => Err(...)
        };
    }
    // existing Linux-only block
}
```

**CI change:** Add `macos-latest` runner to `.github/workflows/release.yml` that
builds and uploads darwin tarballs alongside the existing musl builds.

**FreeBSD/OpenBSD** release upgrades are lower priority since xray/sing-box
binaries don't exist there.

Verification: `xrat upgrade` on macOS downloads and replaces itself correctly.
