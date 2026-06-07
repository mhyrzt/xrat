# 01.1 Trivial, P3: Fix ping flags for FreeBSD/OpenBSD

**Difficulty:** Trivial — 5 minutes, one file.

**File:** `src/prober/icmp/mod.rs:104-113`

The `else` fallback uses Linux `-W` flag, which is wrong on FreeBSD (`-t`) and
OpenBSD (`-w`). Add explicit branches:

```rust
fn ping_flags() -> (&'static str, &'static str) {
    if cfg!(target_os = "macos") {
        ("-c", "-t")
    } else if cfg!(target_os = "linux") {
        ("-c", "-W")
    } else if cfg!(target_os = "freebsd") {
        ("-c", "-t")
    } else if cfg!(target_os = "openbsd") {
        ("-c", "-w")
    } else if cfg!(target_os = "windows") {
        ("-n", "-w")
    } else {
        ("-c", "-W") // best-effort fallback
    }
}
```

Or use `cfg!(any(target_os = "macos", target_os = "freebsd"))` to group BSDs
that share `-t`.

**Verification:** `cargo test` + manual ping on each target.

No library needed.
